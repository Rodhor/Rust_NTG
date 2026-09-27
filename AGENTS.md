# AGENTS.md - Project Guidelines

## Project: Rust_NTG

A Rust web application (v0.1.0) built with Axum that provides a REST API for managing SFTP connections, database queries, and file I/O — no frontend; tested directly via Bruno.

## Current State

Single crate (`rust-ntg`, edition 2024, axum 0.8, ssh2 0.9), ~1230 lines, 10 direct dependencies. **Phase 1 of 9 in progress** — see `Docs/roadmap.md`.

- **Done**: server bootstrap, config/logging, error envelope, `ApiQuery` extractor, SFTP connection with host-key pinning, a 2-session `SessionPool`, all 8 SFTP endpoints (3 directory, 5 file), path safety, graceful shutdown including SFTP disconnect.
- **Not started**: CSV handling, frontend.
- `cargo build` emits **6 warnings, all expected** — unused `DatabaseConfig::pool_size` and unused `Paginated`/`Pagination`, both waiting on later phases. Do not "fix" them by deleting the types; they are consumed later.
- A throwaway SFTP server for development lives in `docker/`. Start it with `newgrp docker -c 'docker compose -f docker/docker-compose.yml up -d'`.

> **The database layer is mid-transition as of 2026-09-27.** Two decisions were
> reversed and the code has not caught up:
>
> - **Deployment**: app + database on a **Linux server**; the operator's browser
>   is on Windows and reaches the app through **nginx**, which authenticates and
>   terminates TLS. `APP_BIND_ADDR` stays `127.0.0.1` — nginx is the trust
>   boundary, not a wider bind. See `Docs/deployment.md`.
> - **Database**: **PostgreSQL**, chosen before the first migration was written
>   because a migration cannot be edited later. `src/db/connection.rs` still
>   builds a `SqlitePool` and `Cargo.toml` still enables `features = ["sqlite"]`.
>   The full dialect list is in `Docs/data-model.md`.
>
> **The code is SQLite; the decision is PostgreSQL.** Do not let either document
> lie about the other — check the source. Both reversals came from facts that
> were cheap to verify and expensive to assume, so verify before trusting a
> documented constraint.

**Read `Docs/codebase.md` before doing anything that touches the code.** It holds the source map, the full config key list, the route table, the error taxonomy, nine load-bearing invariants, and the landmines that have already cost debugging time.

## Development Approach

This project is **a learning exercise** — the user wants to understand the concepts, decisions, and reasoning behind the code, not just receive implementations. Assume the user may not know the tools involved; explain unfamiliar concepts rather than assuming prior knowledge.

- **The user writes all code.** The assistant's job is to explain, plan, review, and answer questions — not to edit source files. The user implements every change themselves unless they explicitly ask the assistant to write a specific piece of code.
- **Explain the "why"** behind every design/code decision: reasoning, trade-offs, alternatives considered.
- **Make sure concepts are understood, not just copied** — even the code you write.
- **Research before answering.** Use the latest documentation and current best practices; never guess.
- **Always use skills when they fit.** Before and during work, check the available skills and load any whose description matches the task (e.g. `rust-skills` for Rust coding/review, `api-and-interface-design` for endpoint design, `code-review-and-quality` before merging). Do not skip a matching skill.

## Error Handling

- Use `AppError` (`crate::domain::error`) as the return error type for all fallible code — API handlers, startup, and internal modules. Avoid `Box<dyn std::error::Error>` in signatures.
- When a subsystem gains a new failure mode, add a variant to `AppError` with `#[from]` / `#[error(...)]` (thiserror) and cover it in `status()` / `code()` — keep the error source chain intact rather than stringifying.

## Avoiding Feature Creep

- Question requests that aren't strictly needed now: is it needed, or can it wait?
- If a feature is deferred, record it in `Docs/` (with a summary in `Docs/overview.md`) for later evaluation — do not build it now.
- `Docs/` holds three kinds of document, distinguished by their status line: **In force** (decisions to follow and reference), **Deferred / Superseded** (forward-looking, not current requirements), and **Living** (maps of the code as it actually is). Start at `Docs/codebase.md` for orientation, then `Docs/overview.md` for why, then `Docs/roadmap.md` for the phase plan and `Docs/open-decisions.md` for anything still undecided.

## SFTP Path Safety

The SFTP layer is the security boundary of this project. Before changing anything
in `path_helpers.rs`, `sftp/file_operations.rs`, or a file handler, read the
Invariants section of `Docs/codebase.md`. The short version:

- **Client input becomes a path only through `path_helpers`.** Never `join`, `push`, or concatenate a query parameter onto a root. `resolve_path` rejects absolute paths and any component that is not `Normal` or `CurDir`.
- **`require_path` guards every remote path field** except `dir/list`. `resolve_path` maps `""` to the root, which is correct for listing and wrong everywhere else — an upload's temp sibling would land *outside* the root, and `dir/delete` would `rmdir` the root.
- **Never call `sftp.*` on a local path, or `std::fs` on a remote path.** This has been gotten wrong in both directions and failed silently both times.
- **The remote root is never client-supplied.** It comes from `SFTP_REMOTE_ROOT_DIR`. A handler that accepts a root as a parameter is a jail escape.

## Communication Style

- Short, concise, and easy to understand. The user will ask questions if something is unclear.
- Precise and actionable in recommendations.
- Always reference official, up-to-date documentation (Rust, Axum, etc.).
- Use examples to explain concepts — close to, but not necessarily exactly, the needed code.
- Keep things simple; don't overcomplicate what can stay simple.