# Codebase Map

> **Status: Living document. Updated whenever the shape of the code changes.**
>
> **Purpose:** orientation for a reader picking up the project cold — a human
> returning after a break, or an agent that would otherwise have to read all
> 1200 lines to find one function.
>
> This file answers **where things are** and **what must not break**. For **why**
> things were decided, see [overview.md](overview.md); for the **plan**, see
> [roadmap.md](roadmap.md). It is not a substitute for reading the code before
> changing it — it is a substitute for reading all of it.

## What this project is

A Rust/Axum HTTP service that moves files between an external SFTP server and
the local filesystem, and will eventually hold a SQLite ledger of items imported
from ERP CSV exports. No frontend yet; the API is exercised directly with Bruno or
curl. It is a learning project, so the reasoning behind a decision matters as much
as the decision.

Single crate `rust-ntg`, edition 2024, ~1230 lines of Rust in 31 files, 10 direct
dependencies. Phase 1 of 9 is in progress: the SFTP layer is complete and the
database layer has not been started.

## Commands

```bash
cargo build                  # 6 warnings, all known dead code (see Current state)
cargo clippy --all-targets   # clean apart from the same dead code + 1 needless borrow
cargo fmt

# Run against the Docker SFTP fixture (must already be up — see below)
cargo run
```

The service binds `127.0.0.1:8080` and reads configuration from `.env` at
startup. `.env` is gitignored and holds the SFTP password and host key
fingerprint; `.env.example` is the committed template and must be kept in sync
with `SftpConfig` in `domain/config_model.rs`.

### Docker SFTP fixture

A throwaway SFTP server for development. Nothing about the real server is
simulated differently, so a change that works here works there.

```bash
newgrp docker -c 'docker compose -f docker/docker-compose.yml up -d'   # first time in a shell
docker compose -f docker/docker-compose.yml ps
./docker/sftp-fingerprint.sh        # current host key, 64 hex chars, no "SHA256:" prefix
```

It listens on `127.0.0.1:2222` with a fixed keypair generated at image build time,
so the fingerprint is stable for as long as the image is not rebuilt. When the
image *is* rebuilt (or `--build --no-cache` is used) the fingerprint changes and
`SFTP_HOST_KEY_SHA256` in `.env` must be updated to match. The script exists
because deriving it by hand is error-prone — the fingerprints that ssh prints are
base64, the one the app wants is hex.

## Source map

| Path | Responsibility |
| --- | --- |
| `main.rs` | Startup order: dotenv → config → logging → `create_dir_all(local_root_dir)` → router → bind → serve → drain → disconnect. 45 lines, and the order matters. |
| `state.rs` | `AppState { config, sftp: Arc<SessionPool> }`, the only shared state. |
| `router.rs` | `build_router`, the `/api/v1` nest, the request-logging layer, and `/health`. |
| `shutdown.rs` | One function: await ctrl-c. Does not own the drain — see Invariants. |
| `extract.rs` | `ApiQuery<T>`, a `FromRequestParts` wrapper that converts Axum's query rejection into `AppError::InvalidQuery` so errors keep the JSON envelope. |
| `path_helpers.rs` | `resolve_path`, `require_path`, `sibling_with_suffix`. The path safety layer, and the most security-relevant code in the project. |
| `middleware.rs` | `log_requests` via `tracing`. |
| `config.rs` | Thin wrapper over `domain/config_model.rs`. |
| `logging.rs` | `tracing` + `tracing-appender`; level, target, and format from `LogConfig`. |
| `domain/config_model.rs` | Typed config blocks (`app`, `log`, `sftp`, `database`), `envy` deserialization, and `validate()`. 247 lines. |
| `domain/error.rs` | `AppError` (11 variants) with `status()` and `code()`, and the JSON `ErrorBody` envelope. |
| `domain/response.rs` | `ApiResponse<T>` (always 200) and the not-yet-used `Paginated`/`Pagination`. |
| `domain/file_item.rs` | `FileItem` — the listing response shape. |
| `domain/health.rs` | `Health` response shape. |
| `sftp/connection.rs` | `connect_blocking` / `disconnect_blocking`, and `verify_host_key`. All libssh2 work happens in `spawn_blocking`. |
| `sftp/pool.rs` | `SessionPool`: a `Vec<Session>` free list behind a `Semaphore` permit, plus `with_session` and `disconnect_all`. |
| `sftp/file_operations.rs` | `upload_file`, `download_file`, `delete_remote_file`, `move_remote_file`. The staging-file pattern lives here. |
| `sftp/directory_operations.rs` | `list_dir`, `create_remote_dir`, `delete_remote_dir`, including hidden-file filtering and sorting. |
| `api/sftp/router.rs` | The eight SFTP routes. |
| `api/sftp/file_handler.rs` | Query structs and handlers for the five file endpoints; `local_target` resolves local paths. |
| `api/sftp/dir_handler.rs` | Query structs and handlers for the three directory endpoints. |
| `api/csv/`, `api/operations/` | Route stubs only — no handlers yet. |
| `csv/mod.rs`, `db/mod.rs` | Empty. Phase 1 and Phase 6 respectively. |

## Configuration

All keys live in `domain/config_model.rs` and are prefixed per block. `validate()`
runs at startup and is the only gate — an unvalidated field reaches the SFTP layer.

| Key | Default | Notes |
| --- | --- | --- |
| `APP_NAME`, `APP_BIND_ADDR`, `APP_URL` | `Rust NTG`, `127.0.0.1:8080` | The bind is loopback **by decision**, not by convenience — see [api-bind-address.md](api-bind-address.md). |
| `LOG_LEVEL`, `LOG_TARGET`, `LOG_FORMAT`, `LOG_FILE_PATH` | `debug`, `file`, `pretty`, `logs/app.log` | `LOG_TARGET=file` makes the container's healthcheck output noisy; that is expected. |
| `SFTP_HOST`, `SFTP_PORT`, `SFTP_USERNAME`, `SFTP_PASSWORD` | `22` | Password is required; `validate()` rejects a missing one. |
| `SFTP_REMOTE_ROOT_DIR` | `./` | The SFTP jail. Every remote path is resolved inside it. |
| `SFTP_LOCAL_ROOT_DIR` | `./data/sftp-transfer` | The local base. Every `local_path` is resolved inside it, and `main` creates it at startup. |
| `SFTP_IN_DIR`, `SFTP_OUT_DIR`, `SFTP_ARCHIVE_DIR` | `in`, `out`, `archive` | Optional, all default `None`. For the Phase 4 import workflow, not used yet. |
| `SFTP_CONNECT_TIMEOUT_SECS`, `SFTP_POOL_SIZE` | `10`, `2` | The pool is deliberately small. One session per in-flight request, so size is a concurrency limit. |
| `SFTP_HOST_KEY_SHA256` | — | 64 hex characters, no `SHA256:` prefix. **Mandatory**, and validated for length and hex. |
| `DATABASE_URL`, `DATABASE_POOL_SIZE`, `DATABASE_MAX_CONNECTIONS` | `sqlite:///app.db`, `1`, `1` | Unused. Phase 1. |

`local_root_dir` must be a **relative** path. It is created by `main` as a
non-root user, so an absolute path at the filesystem root fails startup with
`EACCES`.

## HTTP surface

All under `/api/v1`, all responses wrapped in `ApiResponse<T>`, all errors in
`ErrorBody`. Query parameters only — no request bodies.

| Method | Path | Parameters |
| --- | --- | --- |
| GET | `/api/v1/health` | — |
| GET | `/api/v1/sftp/dir/list` | `path` (empty = the root) |
| POST | `/api/v1/sftp/dir/create` | `path` |
| DELETE | `/api/v1/sftp/dir/delete` | `path` |
| POST | `/api/v1/sftp/file/upload` | `remote_path`, `local_path` |
| POST | `/api/v1/sftp/file/download` | `remote_path`, `local_path` |
| DELETE | `/api/v1/sftp/file/delete` | `path` |
| POST | `/api/v1/sftp/file/move` | `from`, `to`, `overwrite` (default false) |

Every query struct field is `#[serde(default)]`, so an omitted field is an empty
string rather than an error. `require_path` is what turns the ones that must not
be empty into `400` instead of a confusing `502`. An empty `path` on `dir/list` is
the one deliberate exception.

## Error taxonomy

`AppError` in `domain/error.rs` is the return type for all fallible code —
handlers, startup, and internal modules. `status()` and `code()` are two separate
matches over the same enum and must be kept in step.

| Variant | Status | Code |
| --- | --- | --- |
| `Config` | 500 | `CONFIG_ERROR` |
| `Sftp(String)` | 502 | `SFTP_ERROR` |
| `Internal` | 500 | `INTERNAL_ERROR` |
| `Io` | 500 | `IO_ERROR` |
| `Logging` | 500 | `LOG_INIT_ERROR` |
| `Dotenv` | 500 | `DOTENV_ERROR` |
| `Ssh2` | 502 | `SSH2_ERROR` |
| `SpawnBlocking` | 500 | `SPAWN_BLOCKING_ERROR` |
| `HostKeyMismatch` | 502 | `HOST_KEY_MISMATCH` |
| `PathEscapesRoot` | 400 | `PATH_ESCAPES_ROOT` |
| `InvalidQuery` | 400 | `INVALID_QUERY` |

`Ssh2` is 502 because an SFTP-layer failure is an upstream problem, not ours.
There is deliberately no 404: `ssh2::Error` has private fields and is not `Clone`,
so a missing file cannot be distinguished inside `status(&self)`. See Landmines.

## Invariants

These are load-bearing. Each one exists because its absence caused a real bug.

1. **Client input becomes a path only through `path_helpers`.** Never `PathBuf::join`
   or string concatenation on a query parameter. `resolve_path` rejects absolute
   paths and every component that is not `Normal` or `CurDir`.

2. **`require_path` guards every remote path field.** `resolve_path` maps `""` to
   the root, which is correct for listing and wrong everywhere else. Without the
   guard: an upload's temp sibling would be written *outside* the root
   (`/incoming` → `/incoming.tmp`), and `dir/delete` with no `path` would
   `rmdir` the root itself.

3. **`sibling_with_suffix` is only safe on a path that has a file name.** It uses
   `with_file_name`, which replaces the last segment; given a directory it
   escapes to the parent. Invariant 2 is what guarantees this. `path.join(".tmp")`
   would be worse still — on `/incoming/a.csv` it yields `/incoming/a.csv/.tmp`.

4. **The staging pattern is: unlink stale → create with `O_EXCL` → copy → `drop` →
   rename, and unlink again if the rename fails.** In `ssh2` terms:
   `open_mode(OpenFlags::WRITE | OpenFlags::EXCLUSIVE)` for the remote and
   `File::options().create_new(true)` for the local. `O_EXCL` refuses to open an
   existing path, so a planted symlink cannot redirect a write. Unlinking a stale
   sibling first is what stops a crashed run from wedging that path forever.

5. **`drop(handle)` before the rename is required.** `ssh2::File::flush()` is a
   no-op, but `write()` calls `libssh2_sftp_write` synchronously and `Drop` issues
   `libssh2_sftp_close_handle`, so nothing is buffered locally — and the handle
   must be closed before it can be renamed.

6. **Never call `sftp.*` on a local path, or `std::fs` on a remote path.** This
   has been gotten wrong twice, in both directions, and in both cases the failure
   was silent. `sftp.unlink` on a local path sends it to the server and cleans up
   nothing here.

7. **All libssh2 work runs inside `spawn_blocking`.** `ssh2::Session` is
   synchronous and blocking. `SessionPool::with_session` moves the session into
   the blocking task and back out in the return tuple, so every path — `Ok`,
   `Err`, and a panic inside the closure — returns the connection correctly
   without a manual "put it back" call that could be forgotten.

8. **Graceful shutdown drains the server first, then the pool.** `axum::serve`
   with `with_graceful_shutdown` does not return until every in-flight request
   has finished, so when it returns no session is checked out and
   `SessionPool::disconnect_all` can drain the free list without racing. Do not
   move the disconnect into `shutdown.rs` — that future resolves at the *start* of
   the drain, when sessions are still in use.

9. **Host key pinning is mandatory and not optional config.** The SFTP server is
   outside our network, so a wrong or unpinned host key is the only thing standing
   between this service and a credentialed proxy. `validate()` rejects a
   fingerprint that is not 64 hex characters.

## Landmines

Each of these cost real debugging time and none of them is visible from reading
the code.

- **`set_keepalive` must come after `handshake()`.** Before the handshake it
  makes libssh2 1.11.1_DEV fail key exchange with a misleading
  `Error: Unable to exchange DH keys` / bad-SHA2-packet error. The current order in
  `connection.rs` is correct — do not "tidy" it.

- **`.gitignore` patterns cannot start with `./`.** `./data/*` silently does
  nothing — `git check-ignore` reports the file as not ignored, and the next
  `git add -A` commits downloaded data. Use `/data/`.

- **Supplementary groups are fixed at login.** Adding the user to the `docker`
  group does not affect a shell that is already running. Use `newgrp docker`, or
  log out and back in. `systemctl is-active dockerd` reporting inactive is a
  red herring — `docker.socket` is socket-activated and the daemon starts on
  first use.

- **`ssh2::Error` cannot be inspected.** Its fields are private and it is not
  `Clone`, so `status(&self)` holds a `&ssh2::Error` it cannot read. `From<ssh2::Error>
  for io::Error` does exist and does map `SSH_FX_NO_SUCH_FILE` to
  `ErrorKind::NotFound`, but it consumes the error, so it cannot be used as a
  fallback. This is the reason there is no 404. To add one, match the error at the
  call site where it is still owned and add a new `AppError` variant.

- **`OpenFlags::EXCLUSIVE` and `create_new` are the reason a retry works.** They
  turn a corrupted temp file into a clean failure, but only because the surrounding
  unlinks clean up first. Removing an unlink to "simplify" reintroduces the
  sticky-failure mode where a path is permanently unusable after a crash.

- **`fs::rename` overwrites on Linux and fails on Windows** when the destination
  exists. Harmless now, will matter in Phase 9.

- **`ssh2::Session`'s `Drop` only calls `libssh2_session_free`.** It does not send
  an SSH disconnect, which is why `disconnect_all` exists rather than relying on
  the drop.

## Current state

Phase 1 in progress. `cargo build` emits 6 warnings, all of them placeholders
waiting for a later phase rather than defects:

- `DatabaseConfig::{url, pool_size, max_connections}` and the `Config::database`
  field are never read — Phase 1.
- `Paginated`, `Pagination`, and `Paginated::new` are never constructed — needed
  for the item list in Phase 5.

Known, deliberate, and not yet fixed:

- `FileItem.path` is an absolute remote path. Whether the API should expose
  absolute or root-relative paths is undecided.
- `resolve_path` is lexical and does not resolve symlinks, so a symlink *inside*
  the root can point outside it. Acceptable while the SFTP account only ever
  writes into the root; revisit if that changes.
- Temp file names are predictable. Safe because of invariant 4, but a `uuid`
  suffix would remove the reliance. Phase 1 adds the dependency.
- The frontend cannot reach a loopback-bound API from another machine. Resolved by
  the single-VM decision, but it is the tripwire to watch in
  [api-bind-address.md](api-bind-address.md).
- `src/db/mod.rs` and `src/csv/mod.rs` are empty. `sqlx`, `csv`, `sha2`, and
  `uuid` are not yet dependencies.

## Where to look

| To change | Start at |
| --- | --- |
| An endpoint's path or params | `api/sftp/router.rs`, then the handler |
| What a path is allowed to be | `path_helpers.rs` — all three functions |
| How a file is written or moved | `sftp/file_operations.rs` |
| Listing, filtering, sorting | `sftp/directory_operations.rs` |
| An error's status or code | `domain/error.rs` — `status()` and `code()` together |
| A configuration key | `domain/config_model.rs`, plus `.env.example`, plus `validate()` |
| Connection, auth, host key | `sftp/connection.rs` |
| Concurrency limits | `sftp/pool.rs` and `SFTP_POOL_SIZE` |
| Startup order or shutdown | `main.rs`, then `sftp/pool.rs::disconnect_all` |
| The error envelope or success wrapper | `domain/response.rs` |
| Why something was decided | [overview.md](overview.md) |
| What to build next | [roadmap.md](roadmap.md) |
| What is still undecided | [open-decisions.md](open-decisions.md) |
