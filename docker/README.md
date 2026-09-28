# Local fixtures

Two throwaway containers for developing against this project: a **PostgreSQL**
to develop the database layer against, and an **SFTP server** to verify the API's
SFTP path.

Test fixtures only. Nothing here belongs in a real deployment.

---

# PostgreSQL

A throwaway PostgreSQL 18 for writing migrations, running `cargo sqlx prepare`,
and throwing the whole schema away cheaply.

## Start it

```bash
cd docker
docker compose up -d postgres
```

Wait for healthy (a first start initialises the cluster, so a few seconds):

```bash
docker compose ps postgres     # STATUS should reach (healthy)
```

The connection string for your project `.env`:

```
DATABASE_URL=postgres://rust_ntg:rust-ntg-dev-password@127.0.0.1:5432/rust_ntg
```

Confirm it from the host before writing any code:

```bash
psql "$DATABASE_URL" -c 'select current_user, version()'
```

## The app role is deliberately not a superuser

`postgres-init.sql` runs once, on first start, and creates an **ordinary**
role. Verified:

```
rust_ntg|f|f|f|f      -- rolsuper, rolcreatedb, rolcreaterole, rolbypassrls
```

Pointing the app at `POSTGRES_USER` instead would be simpler and would quietly
hand the app superuser powers in development. A migration could then use a
superuser-only feature, pass here, and fail in production where the role is
ordinary — a fixture that cannot tell those two cases apart. Same reasoning as
the SFTP section below: make the failure visible.

**The `public` schema grant is load-bearing.** PostgreSQL 15 removed the default
`CREATE` grant on `public`, so a fresh role fails its first migration with
`ERROR: permission denied for schema public` — which reads like a bug in the
migration and is actually a PostgreSQL 15 default. Verified by creating a role
without the grant and watching it fail.

## Two version-specific traps

Both of these were hit while setting this up, and both fail in ways that do not
name the cause:

| Trap | Symptom | Fix |
|---|---|---|
| Data directory moved in PG 18 | Container refuses to boot: *"there appears to be PostgreSQL data in: /var/lib/postgresql/data (unused mount/volume)"* | Mount the volume at `/var/lib/postgresql`, **not** `.../data` |
| Init script runs once | Change `postgres-init.sql`, nothing happens | `docker compose down -v` then `up -d` |

## Reset it

```bash
docker compose down -v      # the -v is what discards the database
```

`-v` matters more here than for SFTP. A half-applied migration in a volume
survives a plain `down`, and the next `up` inherits it. If a migration goes
wrong, `down -v` is the reset.

The volume is disposable by design. Delete the `postgres-data` entry in
`docker-compose.yml` and the data just lives in the container.

## If the app cannot connect

```bash
docker compose logs postgres
```

Three causes, in the order they actually happen:

1. **Not healthy yet** — the first start runs `initdb`. Use
   `docker compose up -d --wait postgres` to block until it is.
2. **Wrong password** — the app password is `rust-ntg-dev-password`, hardcoded
   in `postgres-init.sql`. It is not wired to the compose environment, so
   changing the compose password will not change it.
3. **`SQLX_OFFLINE=true` with an empty `.sqlx/`** — the macros read the cache,
   find nothing, and fail to compile. Unset it until the first
   `cargo sqlx prepare`.

---

# Local SFTP fixture

A throwaway SFTP server for verifying the four things the API's SFTP path has to
get right: **host key pinning, authentication, the path jail, and the session
pool.**

Password auth, host key baked into the image, a
world-readable directory outside the root specifically so a broken jail is
visible. Nothing here belongs in a real deployment.

## Start it

```bash
cd docker
docker compose up -d --build
./fingerprint.sh
```

`fingerprint.sh` prints the fingerprint in the exact `SFTP_HOST_KEY_SHA256`
format your `Config::validate` requires (64 lowercase hex chars — verified
against `ssh-keygen`'s own base64 output and an independent `sha256sum` of the
key blob).

Copy what it prints into your project `.env`:

```bash
SFTP_HOST=127.0.0.1
SFTP_PORT=2222
SFTP_USERNAME=dev
SFTP_PASSWORD=dev-password
SFTP_ROOT_DIR=/incoming
SFTP_HOST_KEY_SHA256=...      # from fingerprint.sh
```

> If the pin is wrong you don't have to guess: the `HostKeyMismatch` response
> body contains the fingerprint the server actually presented, in the same hex
> format. Copy it from there.

Then:

```bash
cargo run
```

## Fixture layout

Everything is under `/incoming` on the server. Paths in the API are **relative
to it**.

```
/incoming/                        <- SFTP_ROOT_DIR, the jail
├── readme.txt
├── .hidden                      <- proves show_hidden works
├── subdir/
│   ├── nested.txt
│   └── .hidden-nested
└── (end of the jail)

/srv/sftp-private/
└── secret.txt                   <- world-readable, OUTSIDE the jail
```

`/srv/sftp-private` is world-readable **on purpose**. If the app's path jail
were broken, a traversal request would succeed and return `secret.txt`. If the
jail works, the request is rejected before the server is ever consulted. A
traversal target that was already unreadable would make a broken jail and a
working one look identical.

## What to verify

Assumes `cargo run` is up and the `.env` above is set. `jq` is assumed for
readability; drop it if you don't have it.

### 1. Host key pinning — the real one first

```bash
# correct fingerprint -> 200
curl -s -o /dev/null -w '%{http_code}\n' \
  'http://127.0.0.1:8080/api/v1/sftp/list?path=subdir'
```

Break the pin deliberately: set `SFTP_HOST_KEY_SHA256` to 64 `a`s, restart, and
confirm you get `502` with code `HOST_KEY_MISMATCH` — and that the body carries
the real fingerprint. That error message is the recovery path if the pin ever
goes stale.

Then restore the real one.

### 2. Authentication

```bash
# wrong SFTP_PASSWORD -> 502 SFTP_ERROR, never a directory listing
```

Set `SFTP_PASSWORD=wrong`, restart, call the endpoint. If you get a listing
back, auth is not being enforced.

### 3. The path jail

```bash
for p in '..' '../sftp-private' '../../etc/passwd' '/etc/passwd' './etc'; do
  printf '%-20s ' "$p"
  curl -s "http://127.0.0.1:8080/api/v1/sftp/list?path=$p"
  echo
done
```

Every one of these must be `400` with code `PATH_ESCAPES_ROOT`, **including the
last one** — a leading `./` is a `CurDir` component and the current
`resolve_path` rejects it. And the control case must still work:

```bash
curl -s 'http://127.0.0.1:8080/api/v1/sftp/list?path=subdir'   # 200
curl -s 'http://127.0.0.1:8080/api/v1/sftp/list?path=subdir/nested.txt'  # not a directory
```

> **Known gap — the root directory itself is unreachable.** `resolve_path`
> rejects empty, `.`, and `./`, and requires every component to be `Normal`. So
> the shortest accepted input is one real child (`subdir`), which means you
> cannot list `/incoming` at all. That's the most likely first request a client
> will make. The fix is small: `CurDir` and empty cannot escape a root, so
> allowing them is safe — only `ParentDir`, `RootDir`, and `Prefix` need
> rejecting.
>
> ```rust
> // drop the empty check; it only improves the error message, not the outcome
> if relative
>     .components()
>     .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
> ```
>
> `"."` then yields `root.join(".")` and `""` yields `root.join("")`, both the
> root itself. Still no escape route.

### 4. Hidden files

```bash
curl -s 'http://127.0.0.1:8080/api/v1/sftp/list?path=subdir' | jq -r '.data[].name'
```

`nested.txt` appears, `.hidden-nested` does not. The handler hardcodes
`show_hidden = false`, so this confirms the filter, not the flag plumbing.

### 5. The pool

```bash
for i in $(seq 1 6); do
  curl -s -o /dev/null -w '%{http_code} ' \
    'http://127.0.0.1:8080/api/v1/sftp/list?path=subdir' &
done; wait; echo
```

Six concurrent requests against `SFTP_POOL_SIZE=2`. All must return `200`.

**What this does and does not prove:** it proves the pool doesn't deadlock or
drop connections under concurrency. It does *not* prove exclusive checkout —
`pool_size=1` would also return six `200`s, just slower. Distinguishing them
needs a timing measurement or a log line in `with_session`, neither of which
exists. If you want that proof, the cheapest version is a
`tracing::debug!` in `with_session` showing checkout/return and the free-list
length.

## Look at the server side

```bash
docker compose logs -f sftp     # sshd at LogLevel VERBOSE, shows auth attempts
```

Useful when auth fails — it says whether the failure was the password, the
account, or a key the server never offered.

One line in there is expected noise: every healthcheck probe opens port 22 and
closes immediately, so sshd logs `kex_exchange_identification: Connection closed
by remote host` and `Disconnected from 127.0.0.1 [preauth]` a few times a
minute. That is the healthcheck, not an attack and not a client of yours. Real
client traffic arrives from the Docker bridge address (`172.20.0.1`), not
`127.0.0.1`.

## If the API returns "Unable to exchange encryption keys"

```
{"error":{"code":"SSH2_ERROR","message":"ssh2: [Session(-8)] Unable to exchange encryption keys"}}
```

This one is **not** the fixture's fault and not a host key problem — the
handshake negotiated `curve25519-sha256` + `ssh-ed25519` and then died waiting
for the server's key-exchange reply, less than a millisecond in. The server is
provably fine: `ssh` inside the container reaches the auth stage, and a libssh2
client authenticates and reads `/incoming` without trouble.

The cause is `set_keepalive` with a nonzero interval being called *before*
`handshake()`. libssh2 `1.11.1_DEV`, as vendored by `libssh2-sys 0.3.3`, fails
100% of the time in that order:

| call | before `handshake()` | after `handshake()` |
|---|---|---|
| no `set_keepalive` | works | works |
| `set_keepalive(_, 0)` | works | works |
| `set_keepalive(_, <any nonzero>)` | **fails** | works |

So move the `set_keepalive` call below `session.handshake()`. The interval value
does not matter, and the ordering is the whole fix. This is a libssh2 defect, not
misuse: the vendored source only reads `keepalive_interval` inside
`libssh2_keepalive_send`, which nothing calls during the handshake.

The `fingerprint.sh` and auth paths are independent of this, so if you are
debugging the pin and hit this error, the pin was never actually tested.

## Inspect it with a real client

Same image, different job, so `up` stays at one container:

```bash
docker compose run --rm sftp-client
```

Inside:

```bash
sftp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null dev@sftp
```

The host is `sftp` (the compose service name), port 22 — the client is on the
compose network, so it does not go through the published `2222`. It prompts for
the password (`dev-password`); `sftp -b` will not, so batch mode cannot be used
to script this.

This is the way to confirm what the server actually exposes, independent of what
the app claims about it. In particular, `cd /srv/sftp-private && ls` will
succeed — the server imposes no chroot, which is the point: the jail is the
application's job, and the fixture makes the app's failure visible.

## Teardown

```bash
docker compose down -v
```

The host key is baked into the image, not the container, so the fingerprint
survives `down`/`up` and changes only on rebuild. If you change
`SFTP_USER`, `SFTP_PASSWORD`, or `ROOT_DIR`, rebuild:

```bash
docker compose build --no-cache
```
