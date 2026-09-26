# Docs Overview

> **Purpose:** Architecture decisions, design rationale and the implementation
> plan for this project.
>
> This folder has two kinds of document: **Current** — decisions in force, to be
> followed and referenced during implementation. **Deferred** — forward-looking
> notes about things explicitly *not* being built now.

## Current — decisions in force

Read these before implementing anything. Each records the decision, the reasoning,
and the alternatives that were rejected.

| Doc | Topic | Summary |
| --- | --- | --- |
| [deployment.md](deployment.md) | Where it runs, how it is built | Everything runs on the Windows VM: backend, DB, files, UI. On-demand, not a service. Import must catch up, never assume "today's file". The `.exe` is built by GitHub Actions on `windows-latest` and copied over, because `ssh2`'s C dependencies (libssh2, OpenSSL) make cross-compiling from Linux painful. |
| [data-model.md](data-model.md) | Schema and storage | SQLite, five tables. `items` uses a UUID primary key with `ean`/`sku` as indexed columns plus partial unique indexes, because the natural key is undecided. Operational state in columns, source data in one JSON blob per item — the blob is the archive that lets a newly added field be derived later without re-fetching from a transient remote file. The cleaned projection is never stored. WAL mode from day one. |
| [field-mapping.md](field-mapping.md) | Vocabulary, rules, hashing | The split that resolves the central tension: **the ERP header vocabulary is a Rust enum, the instance configuration is database rows.** Headers are emitted from `ErpField::header()` and are never stored, so they cannot be wrong. The user edits which source column feeds which slot, with which ops, in which order. Ops are a closed typed enum — no parser, no `eval`. Hashes are over `\x1f`-separated `(ntg, value)` pairs in slot order, which avoids both map-order nondeterminism and concatenation ambiguity. |
| [import.md](import.md) | SFTP fetch and ingest | The import is the difference between the remote directory and the ledger, so missed days are caught up automatically. Idempotency is mandatory and enforced by `UNIQUE(remote_path, size)`. One transaction per file, so a failure leaves no half-state. One SFTP session for the whole run, not the existing pool. |
| [export.md](export.md) | ERP CSV and picture CSV | Selection is `approved AND last_exported_hash <> content_hash`, so unchanged items are not re-sent. Updates and new items go in the same CSV. Failed picture downloads never block an export. Re-exporting everything is a single `UPDATE`. |
| [api-surface.md](api-surface.md) | Endpoints and conventions | Versioned JSON under `/api/v1`, namespaced so a second tool is additive. Keyset pagination, never `OFFSET`. Bulk endpoints take a filter, not an id list. HTML routes share the same services and differ only in response type. |
| [frontend.md](frontend.md) | UI technology | Server-rendered HTML via `askama` in the existing binary, with the `.exe` opening the browser. Judged against the actual six-screen inventory, five of which are forms and tables. Escalation path is askama → htmx islands → SPA, none of which touch the backend. Not on the critical path. |
| [open-decisions.md](open-decisions.md) | What is still undecided | Twelve open points with recommendations. Item 1 (inactive slots shift-left vs hold-position, against a **positional** ERP import) is the highest-severity one and blocks Phase 6. |
| [roadmap.md](roadmap.md) | The step plan | Phases 0–8 with deliverables and acceptance criteria. Phases 0–7 contain no frontend work. |

## Deferred — explicitly not being built now

| Doc | Topic | Summary |
| --- | --- | --- |
| [sftp-auth-key.md](sftp-auth-key.md) | SFTP key-based auth | Now (decided 2026-09-26): **password auth**, host key pinned. The SFTP server is outside our network, so it is a password-guessing surface and the credential is a reusable plaintext secret. Later: ed25519 key via `ssh-keygen` (a setup step, not app code) plus server-side `authorized_keys` restrictions, then `PasswordAuthentication no`. Also the enabler for the `sftp.exe` shell-out alternative in [deployment.md](deployment.md). |
| [api-bind-address.md](api-bind-address.md) | API exposure and credential delegation | Now (decided 2026-09-26): `127.0.0.1` only — loopback is the entire access-control story, so no API auth or TLS is needed. The contradiction this doc used to carry ("a frontend on a different server cannot reach a loopback-bound API") is **resolved**: everything is on the Windows VM. The tripwire still stands — if the bind is ever widened, the API becomes a credentialed proxy onto an internet-facing SFTP server and auth becomes mandatory in the same change. |
| [fullstack-rust.md](fullstack-rust.md) | Superseded frontend decision | The original 2026-09-24 "API-only, Leptos later" reasoning. **Superseded** by [frontend.md](frontend.md) once a UI was confirmed as a requirement. Kept because the escalation reasoning and the HTMX re-reversal are worth understanding. |

## Status Legend

- **In force** — decided, to be implemented and referenced.
- **Deferred** — recorded intent, not to be acted on.
- **Superseded** — kept for the reasoning, not to be followed.

## Conventions used in these documents

- **"BLOCKING"** in [open-decisions.md](open-decisions.md) means a specific phase
  cannot be implemented correctly until it is answered.
- **Ponytail principle applied throughout:** the smallest thing that works, with
  deliberate shortcuts marked and their ceilings named. No speculative
  abstraction, no scaffolding for a future that has not been specified.
- Rationale is recorded alongside decisions, because this project is a learning
  exercise — the reasoning is the deliverable as much as the code.
