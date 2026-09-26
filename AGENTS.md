# AGENTS.md - Project Guidelines

## Project: Rust_NTG

A Rust web application (v0.1.0) built with Axum that provides a REST API for managing SFTP connections, database queries, and file I/O — no frontend; tested directly via Bruno.

## Current State

Single crate (`rust-ntg`, edition 2024, axum 0.8). Bootstrapped server skeleton:

- **main**: dotenv → config load → logging init → state → router → bind → `axum::serve` with graceful shutdown; returns `AppError`
- **config**: `.env` via `dotenv` + `envy`; `domain/config_model.rs` holds typed prefixed config blocks (`Config { app, log, sftp, database }`)
- **logging**: `logging.rs` — `tracing` + `tracing-subscriber` + `tracing-appender`; level/target/format driven by `LogConfig`
- **domain**: `config_model`, `error` (`AppError` + `ErrorBody`), `response` (`ApiResponse`, `Paginated`), `health`
- **router**: `build_router(state)` → `GET /api/v1/health`
- **state**: `AppState { config: Config }`
- **shutdown**: ctrl+c triggers graceful shutdown (drains in-flight requests)
- **extract / middleware / sftp / csv**: empty stubs; filled in as features land

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
- `Docs/` holds two kinds of document, distinguished by their status line: **In force** (decisions to follow and reference) and **Deferred / Superseded** (forward-looking, not current requirements). Start at `Docs/overview.md`, then `Docs/roadmap.md` for the phase plan and `Docs/open-decisions.md` for anything still undecided.

## Communication Style

- Short, concise, and easy to understand. The user will ask questions if something is unclear.
- Precise and actionable in recommendations.
- Always reference official, up-to-date documentation (Rust, Axum, etc.).
- Use examples to explain concepts — close to, but not necessarily exactly, the needed code.
- Keep things simple; don't overcomplicate what can stay simple.