# AGENTS.md - Project Guidelines

## Project: Rust_NTG

A Rust web application (v0.1.0) built with Axum that provides a REST API for managing SFTP connections, database queries, and file I/O — with a minimal HTMX frontend for testing.

### Architecture (Workspace with 4 crates)

- **ntg-app**: Main Axum web server, routes, API handlers
- **db**: Database connectivity
- **sftp**: SFTP file transfer
- **io**: File read/write operations

## Development Approach

- **User drives all development** — I will not make code changes unless explicitly asked
- I provide guidance, research, and answers to questions
- I will look up documentation before answering, using latest versions
- I do not guess — I research first

## When Providing Guidance

- Always use the latest documentation
- Search for current best practices
- Reference official docs (Rust, Axum, etc.) when explaining
- Be precise and actionable in recommendations
- Use examples to explain concepts — examples should be close but not necessarily the exact code needed
- Keep explanations simple and easy to understand
- Don't make things overly complicated just because it's possible