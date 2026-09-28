# Deployment Target and Build/Delivery

> **Status: Target reversed 2026-09-27, superseding the 2026-09-26 decision.
> In force (target).**
> The app runs on a **Linux server**. The browser runs on Windows and reaches it
> through **nginx on the Linux server**, which terminates TLS and authentication.
> The app itself stays bound to `127.0.0.1`.
> **How** the release is built and delivered is still not decided — see the
> deferred section below.

## What changed, and why

The previous target (everything on the Windows VM) was decided on 2026-09-26 and
rested on one fact: **the VM could already reach the SFTP server**, which was the
unknown that made a split deployment conceivable. That same fact was then asked
about the *other* machine, and it turned out the Linux server reaches both
externals:

| Reachability | Confirmed 2026-09-27 |
| --- | --- |
| Linux server → SFTP server | **Yes** — the inbound CSV is pulled from here |
| Linux server → ERP GraphQL endpoints | **Yes** — items are pushed from here |
| Windows → Linux server | **Yes** — this is how the browser reaches the app |

So the deciding fact did not reverse — it was simply asked in the other direction
and came back the same way. With every external dependency reachable from the
Linux box, keeping the app on Windows bought nothing and cost the ERP integration
(see below). Both servers are under the same control, so the operational cost of
the split is a service file and an nginx config.

**The two things that made the split attractive:**

1. **No item data has to cross between the servers.** The inbound CSV is pulled
   by the Linux server from the external SFTP server, and items are pushed to the
   ERP over GraphQL. The Windows machine is a browser and nothing more. The old
   "the CSV and pictures are on the wrong machine" objection does not apply to
   items.
2. **The app stops competing with the ERP for the same box.** The old target ran
   the app, the database, the ERP and the operator's interactive session on one
   Windows VM, so a heavy import contended with whatever else was running.

**What the split does cost** — honestly, and it is not nothing:

- A database server to install, secure, back up and keep alive, instead of a file
  you can `cp`. See [data-model.md](data-model.md) for why PostgreSQL was chosen
  anyway.
- nginx, TLS certificates, and API authentication — the tripwire in
  [api-bind-address.md](api-bind-address.md) **fires** with this change and is
  satisfied at the proxy.
- Pictures still have to cross the machine boundary, because they are binaries
  and GraphQL is a poor transport for them. Currently expected to be `robocopy`
  from Windows to an SMB share on Linux, which is the one direction the
  reachability table guarantees. Undecided — see
  [open-decisions.md](open-decisions.md).
- Backup is no longer a folder copy. PostgreSQL needs `pg_dump`.

## The Decision

**The application and its database run on a Linux server. The operator's browser
runs on Windows.**

```
  Windows workstation                    Linux server
  ┌──────────────────────┐               ┌─────────────────────────────┐
  │  browser  ───────────┼── HTTPS ─────▶│  nginx  (TLS + auth)        │
  │  (no app, no DB)     │               │    │                        │
  └──────────────────────┘               │    ▼ 127.0.0.1:8080         │
                                         │  rust-ntg (Axum)           │
                                         │    │                        │
                                         │    ├──▶ PostgreSQL 127.0.0.1│
                                         │    ├──▶ SFTP server   (out) │
                                         │    └──▶ ERP GraphQL   (out) │
                                         └─────────────────────────────┘
```

Both machines are controlled by the same operator. There is no CI pipeline and
nothing is delivered to anyone else.

## Why This Target Won

| Option | Verdict |
| --- | --- |
| **App + DB on a Linux server, browser on Windows, nginx in front** | **Chosen (2026-09-27).** The Linux box reaches both externals, so items never need to cross between the machines. App and ERP stop contending for one box. |
| Everything on the Windows VM | **Superseded 2026-09-27.** Was chosen 2026-09-26 on the assumption that keeping items local was worth more than reaching the ERP. Once the Linux box was confirmed to reach both the SFTP server *and* the ERP GraphQL endpoints, the ERP integration stopped being a reason to stay on Windows, and the app no longer shares a machine with the ERP. |
| Backend on a Linux server, browser on the VM, **no proxy** | Rejected. A bare LAN bind puts an unauthenticated, unencrypted, credentialed proxy on the network — exactly what [api-bind-address.md](api-bind-address.md) forbids. nginx is what makes the split safe. |

## Consequences of the Split

1. **`APP_BIND_ADDR=127.0.0.1` still stays.** This is the load-bearing
   consequence. nginx is the trust boundary; the app keeps a loopback socket and
   is unreachable from the LAN except through the proxy, which authenticates and
   encrypts. The tripwire's requirements are met at the proxy rather than deleted.
2. **nginx is mandatory, not a convenience.** It must terminate TLS, and it must
   require authentication before forwarding. Until both exist, the app should not
   be started at all. There is no "or just use the IP directly" variant — that
   reopens exactly the hole the tripwire describes. If nginx is down and a
   fallback is wanted, an SSH tunnel preserves the invariant; a raw bind does not.
3. **The import must still catch up, never "take today's file".** This is
   unchanged and is now the *most* important property of the deployment, because
   a Linux server will be restarted and updated far more often than a VM that
   only runs when the operator launches it. If the server was off for three days,
   three days of files are on the SFTP server and all of them must be picked up.
4. **Backup is no longer `cp`.** `pg_dump` plus the `imports/`, `pictures/` and
   `logs/` directories. The `imports/` directory is not optional: it is what makes
   re-derivation possible if a file is ever removed from the remote directory.
5. **The app is a service, not an on-demand double-click.** A systemd unit, with
   the nightly import triggered by a timer. The old "on-demand, opened by the
   user" shape is gone, and with it the "no in-process daily timer" reasoning
   that justified it — the OS scheduler argument in the previous version of this
   document still holds and is now the *only* scheduler.

## The App Is a Long-Running Service

The app runs continuously under systemd and is started by a timer or restarted
after a deployment, not by a human double-clicking an icon.

### What That Forces

**1. The import is triggered by the OS scheduler, not by app start.** A
`systemd` timer (or `cron`) calls the import endpoint. This keeps the previous
reasoning intact: a `tokio::time::interval` would mean reasoning about missed
runs, clock changes and overlap for no benefit, and the OS scheduler already
handles retries, restart-after-failure and logging.

**2. `SFTP_DEFAULT_OUT_DIRECTORY` / `SFTP_DEFAULT_ARCHIVE_DIRECTORY` semantics
are now more important, not less.** With a service that runs unattended every
night, whatever the app does to consumed files on *someone else's* server happens
without a human watching. See [open-decisions.md](open-decisions.md) item 5.

## Build and Delivery

> **Status: Deferred. Not a current requirement.**
> The previous target needed a cross-compiled Windows `.exe` and a long analysis
> of the `ssh2` → libssh2 → OpenSSL C dependency chain. **That analysis is
> obsolete**: the app is built for and runs on Linux, where `cargo build` is the
> whole story and the C dependency is whatever the distribution provides. The
> remaining question is only how a release binary and a `.env` reach the Linux
> server, which is a `scp` and a service restart. Chosen in
> [roadmap.md](roadmap.md) Phase 9, when there is a finished binary to produce.

### What replaced the Windows cross-compile analysis

| Old concern | Status now |
| --- | --- |
| `x86_64-pc-windows-gnu` + mingw-w64 + Perl | **Dropped.** No Windows binary is produced. |
| Vendoring OpenSSL for an MSVC target | **Dropped.** Same reason. |
| GitHub Actions `windows-latest` | **Dropped**, and there is still no CI. |
| `russh`/`russh-sftp` to shed the C dependency | **Dropped.** The OpenSSL the system provides on Linux is fine. |
| Shell out to a system `sftp` binary | **Dropped**, but see [sftp-auth-key.md](sftp-auth-key.md) — it remains the recorded fallback if the C dependency ever becomes a problem on Linux. |

One genuine simplification follows from this target: the build no longer needs
`cargo sqlx prepare` to be a special case, because the target platform is the
development platform. See [data-model.md](data-model.md).

### Acceptance

- `rust-ntg` runs on the Linux server under systemd, and `GET /api/v1/health`
  answers through `https://<server>` with nginx in front.
- Health reports the database as reachable with a real query, not a local flag.

## Configuration

```ini
# === SERVER ===
APP_BIND_ADDR=127.0.0.1:8080     # stays loopback; nginx is the only client

# === DATABASE ===
DATABASE_URL=postgres://rust_ntg:<password>@127.0.0.1/rust_ntg
DATABASE_MAX_CONNECTIONS=2

# === PATHS ===
IMPORTS_DIR=imports
PICTURES_DIR=pictures
EXPORTS_DIR=exports
```

`DATABASE_URL` rather than a path, because the URL is engine-neutral: it is the
one key that does not change when the database engine does, and it is what lets
the same `.env` work in development and on the server. Note that the URL now
carries a password, so `DatabaseConfig` must redact it in `Debug` — see
[open-decisions.md](open-decisions.md).

`SFTP_DEFAULT_IN_DIRECTORY` / `OUT` / `ARCHIVE` already exist. Whether the app
moves, renames or archives source files **on the SFTP server** after import is
still undecided — see [open-decisions.md](open-decisions.md).

## Related

- [api-bind-address.md](api-bind-address.md) — why the bind stays loopback and
  nginx becomes the trust boundary.
- [data-model.md](data-model.md) — why PostgreSQL rather than a SQLite file, and
  why that choice is a *daily* cost as well as an operational one.
- [open-decisions.md](open-decisions.md) — picture transport, the ERP credential,
  and remote file handling.
- [roadmap.md](roadmap.md) — Phase 9 is where the build route gets chosen.
