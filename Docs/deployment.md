# Deployment Target and Build/Delivery

> **Status: Target decided 2026-09-26, revised 2026-09-27. In force (target).**
> Everything — backend, database, files, UI — runs on the Windows VM.
> **How** the `.exe` gets built is *not* decided — see the deferred section below.
> Development, building, and testing are Linux-only until the project is finished.

## The Decision

**The entire application runs on the Windows VM.** There is no second machine
serving a frontend, and no Linux server in the request path.

The VM:

- reaches the SFTP server (FileZilla already does, from that machine),
- runs the ERP, which is local and exposes an authenticated GraphQL endpoint,
- holds the SQLite database, the picture folder, the export folder.

## Why This Target Won

| Option | Verdict |
| --- | --- |
| **Everything on the Windows VM** | **Chosen.** Removes the auth question, puts the ERP CSV where the ERP already is, keeps all data on one machine. |
| Backend on a Linux server, browser on the VM | Rejected. Forces a LAN bind, which forces auth + TLS (see [api-bind-address.md](api-bind-address.md)), and leaves the CSV and pictures on the wrong machine. |

The deciding fact: **the VM can already reach the SFTP server.** That was the
unknown that made a split deployment conceivable. It is settled.

## Consequences of Everything Being Local

1. **`APP_BIND_ADDR=127.0.0.1` stays.** Loopback is the whole access-control
   story. No API auth, no TLS. The tripwire in
   [api-bind-address.md](api-bind-address.md) is unchanged and still applies.
2. **The ERP CSV is written where the ERP already is.** No download step, no SMB
   share, no artifact transfer.
3. **Backup is a folder copy.** `app.db` + `imports/` + `pictures/` +
   `exports/` + `logs/`. Scriptable with a single `robocopy`/`cp`.
4. **One machine to reason about.** One failure domain, one place to look.

## The App Is On-Demand, Not Always-On

The app does **not** need to survive a reboot on its own. It starts when the
user launches it, or when a scheduled task triggers it. It is not a Windows
service.

### What That Forces

**1. No in-process daily timer.** A `tokio::time::interval` would mean reasoning
about missed runs, clock changes and overlap for no benefit. The OS scheduler
already does retries, reboot handling and logging. Import is triggered by:

- app start (user launches the exe),
- a "Import now" action in the UI,
- Windows Task Scheduler calling the import endpoint, if unattended runs are
  wanted.

**2. The import must catch up, never "take today's file".** This is the single
most important consequence. If the VM was off for three days, three days of files
are sitting on the SFTP server and they must all be picked up.

Therefore: **idempotency is mandatory, not an optimisation.** The `imports`
table is the authority on "what do I already have", keyed on
`(remote_path, size)`. The import is defined as *the difference between what the
remote directory contains and what the ledger records* — not as *a file for
today*. See [import.md](import.md).

**3. Cheapest possible deploy is a single `.exe`.** It binds `127.0.0.1:8080`,
opens the default browser at `http://127.0.0.1:8080`, and keeps its data in a
folder next to itself (or under `%LOCALAPPDATA%`). No installer, no service, no
registry entries, no `Program Files` writes. `Task Scheduler` with "run at logon"
if the user wants it up automatically.

## Build and Delivery

> **Status: Deferred as of 2026-09-27. Not a current requirement.**
> The route is chosen in [roadmap.md](roadmap.md) Phase 9, once there is a
> finished binary to produce. Nothing in this section constrains day-to-day work:
> on Linux the build is just `cargo build`, and the C dependency question does not
> arise. The analysis is kept because it is still the right analysis when the time
> comes.

### The Problem

The app depends on `ssh2`, which is Rust bindings to **libssh2 (C)** and pulls in
**OpenSSL (C)**. `libssh2-sys/build.rs` looks for vcpkg on MSVC targets and
otherwise compiles libssh2 from bundled C source; OpenSSL comes from the system
or from `openssl-src` (needs Perl, and a matching C toolchain). So "build on
Linux, copy a binary over" is not free.

### Options Considered

| | Build on Linux → copy exe | Local effort | Keeps `src/sftp/`? |
| --- | --- | --- | --- |
| A. GitHub Actions `windows-latest` | n/a — real MSVC toolchain, download the artifact | ~15 lines of YAML | **Yes, untouched** |
| B. `cargo-xwin` → `x86_64-pc-windows-msvc` | Vendored OpenSSL for an *MSVC target* from Linux is the risky part | Days if it fails | Yes |
| C. `x86_64-pc-windows-gnu` + `vendored-openssl` | mingw-w64 + Perl; `+crt-static` for a standalone exe | Moderate | Yes |
| D. `russh` + `russh-sftp` (pure Rust) | `ring` on windows-msvc via cargo-xwin has a known open issue; clean on windows-gnu | Rewrite of `src/sftp/` (~150 lines) | No |
| E. Shell out to the Windows-bundled `sftp.exe` | App becomes pure Rust, zero C deps | ~80 lines + output parsing | No |
| F. Install the Rust toolchain on the VM and build there | None — the build simply happens on the target | Minutes, once | Yes |

**No option is chosen.** C is the current front-runner because it keeps the
work on Linux, which is where all of it already happens; F is the reliable
fallback if mingw proves troublesome; A is still the least risky *toolchain* but
costs a CI pipeline for a single consumer, which is not worth it for a prototype.

Option E is the genuinely laziest alternative and is recorded here for the day
the build route stops being acceptable. It dovetails with
[sftp-auth-key.md](sftp-auth-key.md): OpenSSH batch mode is frictionless with a
key and awkward with a password on Windows (it needs `SSH_ASKPASS`), and
`known_hosts` does host-key pinning better than the hand-rolled check in
`src/sftp/connection.rs:38`.

Note `cargo sqlx prepare` will **not** be needed on any of these routes: the
database layer uses sqlx's runtime query API (`sqlx::query` + `FromRow`), not the
compile-time macros. That is a deliberate choice so the build never needs a live
database or a checked-in `.sqlx/` directory.

### Acceptance

- `rust-ntg.exe` runs on the VM, prints its resolved config paths, and answers
  `GET /api/v1/health`.
- The DB, `imports/`, `pictures/`, `exports/` and `logs/` are created relative to
  the working directory on first run.

## Configuration Additions

```ini
# === PATHS ===
APP_DB_PATH=app.db
IMPORTS_DIR=imports
PICTURES_DIR=pictures
EXPORTS_DIR=exports
```

`SFTP_DEFAULT_IN_DIRECTORY` / `OUT` / `ARCHIVE` already exist. Whether the app
moves, renames or archives source files **on the SFTP server** after import is
still undecided — see [open-decisions.md](open-decisions.md).

## Related

- [api-bind-address.md](api-bind-address.md) — why the bind stays on loopback.
- [sftp-auth-key.md](sftp-auth-key.md) — the credential story.
- [frontend.md](frontend.md) — why the UI is served from this same binary.
- [roadmap.md](roadmap.md) — Phase 9 is where the build route above gets chosen.
