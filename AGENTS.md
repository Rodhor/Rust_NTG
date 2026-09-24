# AGENTS.md - Project Guidelines

## Project: Rust_NTG

A Rust web application (v0.1.0) built with Axum that provides a REST API for managing SFTP connections, database queries, and file I/O — with a minimal HTMX frontend for testing.

## Current State

Single crate (`rust-ntg`, edition 2024, axum 0.8). Early-stage scaffold:
`src/main.rs` is a stub `fn main() {}`; modules are declared but largely empty (e.g. `router.rs`, `state.rs`).

- **config**: configuration loading (`.env` via `dotenv` + `envy`)
- **domain**: domain models (e.g. `config_model`)
- **extract / error / middleware / shutdown / state**: Axum plumbing scaffolding
- **router**: routes/API handlers (not yet built)
- **sftp**: SFTP file transfer
- **csv**: CSV handling

## Development Approach

This project is **a learning exercise** — the user wants to understand the concepts, decisions, and reasoning behind the code, not just receive implementations. Assume the user may not know the tools involved; explain unfamiliar concepts rather than assuming prior knowledge.

- **The user drives all development.** Make no code changes unless explicitly asked.
- **Explain the "why"** behind every design/code decision: reasoning, trade-offs, alternatives considered.
- **Make sure concepts are understood, not just copied** — even the code you write.
- **Research before answering.** Use the latest documentation and current best practices; never guess.
- **Always use skills when they fit.** Before and during work, check the available skills and load any whose description matches the task (e.g. `rust-skills` for Rust coding/review, `api-and-interface-design` for endpoint design, `code-review-and-quality` before merging). Do not skip a matching skill.

## Avoiding Feature Creep

- Question requests that aren't strictly needed now: is it needed, or can it wait?
- If a feature is deferred, record it in `Docs/` (with a summary in `Docs/overview.md`) for later evaluation — do not build it now.
- `Docs/` holds forward-looking notes only; do not treat them as current requirements.

## Communication Style

- Short, concise, and easy to understand. The user will ask questions if something is unclear.
- Precise and actionable in recommendations.
- Always reference official, up-to-date documentation (Rust, Axum, etc.).
- Use examples to explain concepts — close to, but not necessarily exactly, the needed code.
- Keep things simple; don't overcomplicate what can stay simple.