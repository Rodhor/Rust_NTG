# Docs Overview

> **Purpose:** Architecture decisions, design rationale and the implementation
> plan for this project.
>
> This folder has two kinds of document: **Current** — decisions in force, to be
> followed and referenced during implementation. **Deferred** — forward-looking
> notes about things explicitly *not* being built now.
>
> **Start with [codebase.md](codebase.md)** if you need to find your way around
> the actual code. It maps the source tree, configuration, routes, error taxonomy,
> invariants, and known landmines. This document answers *why*; that one answers
> *where*.

## Two Reversals, 2026-09-27

Read this first if you have seen an earlier version of these documents. Two
decisions were reversed, and both were driven by the same discovery.

**1. Deployment: the Windows VM → a Linux server.** The app and its database now
run on a Linux server; the operator's browser runs on Windows and reaches the app
through nginx. The old target was rejected for keeping the ERP integration local
to the VM — and the Linux server turned out to reach the ERP's GraphQL endpoints
*and* the external SFTP server, which removes that reason entirely. Consequence:
no item data crosses between the machines, so the app and the ERP stop contending
for one box. See [deployment.md](deployment.md).

**2. Database: SQLite → PostgreSQL.** Chosen **before the first migration was
written**, because a migration is the one artifact that cannot be edited later.
The code cost of switching later was measured at 31 lines; the cost that mattered
was rewriting migration history. See [data-model.md](data-model.md).

Both reversals were driven by facts that were cheap to check and expensive to
assume. That is the lesson worth carrying to the next open question in
[open-decisions.md](open-decisions.md).

> **The code has not caught up yet.** `src/db/connection.rs` still builds a
> `SqlitePool` and `Cargo.toml` still enables the `sqlite` feature. The decisions
> above are in force; the implementation is mid-transition and will be updated
> next. Do not trust either document over the code on this point — check.

## Current — decisions in force

Read these before implementing anything. Each records the decision, the reasoning,
and the alternatives that were rejected.

| Doc | Topic | Summary |
| --- | --- | --- |
| [codebase.md](codebase.md) | Where the code is, and what must not break | Source map, config keys, routes, error taxonomy, nine load-bearing invariants, and the landmines that already cost debugging time. Updated as the code changes. **Currently mid-transition:** the database layer is still `SqlitePool` on disk while the decision is PostgreSQL. |
| [deployment.md](deployment.md) | Where it runs, how it is built | **Reversed 2026-09-27.** The app and its database run on a **Linux server**; the operator's browser runs on Windows and reaches it through **nginx**, which terminates TLS and authenticates. The app stays on `127.0.0.1`, so nginx is the trust boundary. Chosen after confirming the Linux box reaches both the SFTP server *and* the ERP GraphQL endpoints — which means no item data crosses between the machines at all. |
| [data-model.md](data-model.md) | Schema and storage | **PostgreSQL**, six tables. `items` uses a UUID primary key with `ean`/`sku` as indexed columns plus partial unique indexes, because the natural key is undecided. Operational state in columns, source data in one JSON blob per item — the blob is the archive that lets a newly added field be derived later without re-fetching from a transient remote file. The cleaned projection is never stored. Engine chosen before the first migration was written, because a migration cannot be edited later. |
| [field-mapping.md](field-mapping.md) | Vocabulary, rules, hashing | The split that resolves the central tension: **the ERP header vocabulary is a Rust enum, the instance configuration is database rows.** Headers are emitted from `ErpField::header()` and are never stored, so they cannot be wrong. The user edits which source column feeds which slot, with which ops, in which order. Ops are a closed typed enum — no parser, no `eval`. Hashes are over `\x1f`-separated `(ntg, value)` pairs in slot order, which avoids both map-order nondeterminism and concatenation ambiguity. |
| [import.md](import.md) | SFTP fetch and ingest | The import is the difference between the remote directory and the ledger, so missed days are caught up automatically. Idempotency is mandatory and enforced by `UNIQUE(remote_path, size)`. One transaction per file, so a failure leaves no half-state. One SFTP session for the whole run, not the existing pool. **Load-bearing now more than ever:** a Linux server gets restarted and updated far more often than a VM that only runs when launched. |
| [export.md](export.md) | ERP CSV and picture CSV | Selection is `approved AND last_exported_hash <> content_hash`, so unchanged items are not re-sent. Updates and new items go in the same CSV. Failed picture downloads never block an export. Re-exporting everything is a single `UPDATE`. **The transport is now contested** — a GraphQL push is available and would remove the positional-import risk. |
| [api-surface.md](api-surface.md) | Endpoints and conventions | Versioned JSON under `/api/v1`, namespaced so a second tool is additive. Keyset pagination, never `OFFSET`. Bulk endpoints take a filter, not an id list. HTML routes share the same services and differ only in response type. |
| [frontend.md](frontend.md) | UI technology | Server-rendered HTML via `askama` in the existing binary. Judged against the actual six-screen inventory, five of which are forms and tables. Escalation path is askama → htmx islands → SPA, none of which touch the backend. Not on the critical path. **The app logs its URL rather than opening a browser** — the browser is on a different machine. |
| [open-decisions.md](open-decisions.md) | What is still undecided | Sixteen open points with recommendations. Item 1 (inactive slots shift-left vs hold-position, against a **positional** ERP import) is the highest-severity one and blocks Phase 6. Items 13–16 were added 2026-09-27: ERP delivery transport, picture transport, the ERP credential, and the migration strategy. |
| [roadmap.md](roadmap.md) | The step plan | Phases 1–9 with deliverables and acceptance criteria. Phases 1–7 contain no frontend work. Phase 1 now includes compile-time SQLx macros, a `justfile` gate, and the PostgreSQL migration. |

## Deferred — explicitly not being built now

| Doc | Topic | Summary |
| --- | --- | --- |
| [sftp-auth-key.md](sftp-auth-key.md) | SFTP key-based auth | Now (decided 2026-09-26): **password auth**, host key pinned. The SFTP server is outside our network, so it is a password-guessing surface and the credential is a reusable plaintext secret. Later: ed25519 key via `ssh-keygen` (a setup step, not app code) plus server-side `authorized_keys` restrictions, then `PasswordAuthentication no`. Since the app moved to Linux this is **no longer a build-enabler** — it is purely a security upgrade. |
| [api-bind-address.md](api-bind-address.md) | API exposure and credential delegation | The app binds `127.0.0.1` and stays there. Loopback was originally sufficient because everything ran on one machine; since the 2026-09-27 split it is a *stronger* guarantee — the app is unreachable by anything except the nginx proxy on the same host. The tripwire **fired** when the app moved to a server, and is satisfied at the proxy: TLS, authentication and authorisation all happen there. **Binding to the LAN address instead is rejected** — that violates the tripwire rather than satisfying it. An SSH tunnel is the acceptable fallback; a raw bind is not. |
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
