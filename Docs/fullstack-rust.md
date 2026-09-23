# Full Rust Stack — Frontend Migration Path

> **Status: Forward-looking. Not currently relevant — revisit at a later date.**
> This doc records an intended future direction, not something to act on today.

## The Goal

A web application built entirely in Rust:

- **Backend:** Rust (Axum) — stays as the REST API / server. Not subject to this migration.
- **Frontend (now):** Server-rendered HTML built in Rust with HTMX + Askama, served from Axum.
- **Frontend (later):** A rewrite of the frontend in **Leptos** as a single-page application (client-side rendering / WASM).

The backend, including all `db` / `sftp` / `io` logic and the Axum REST routes, is intended to remain untouched for this rewrite. The rewrite should be scoped to the frontend only.

## Intended Path

1. **Phase 1 — now:** Axum + HTMX + Askama. Pages are rendered on the server and consume the app's JSON REST endpoints.
2. **Phase 2 — later:** Rewrite the frontend in Leptos as an SPA (WASM, client-side rendering). The browser app calls the existing REST API over HTTP.
3. **Migration:** The HTMX frontend keeps working as a test harness while the Leptos frontend is built alongside it, then swap when the Leptos version is ready. No backend changes required.

## Leptos Mode Decision: SPA (CSR), not Isomorphic

Isomorphic (SSR + hydration) was considered and rejected for this project:

- **SEO / crawlability** — the largest win of isomorphic SSR. Irrelevant for an internal tool.
- **First-paint performance** — matters on public, content-heavy sites. Irrelevant on an internal network.
- **Server functions (`#[server]`)** — the main selling point of isomorphic Leptos, but they duplicate what this app's REST API already does. Choosing isomorphic would mean either redundant server functions or Leptos pages calling the REST API anyway — i.e. an SPA with extra build complexity.
- **Build complexity** — isomorphic requires `cargo-leptos`, two compile targets (server + WASM hydrate), and hydration plumbing. An SPA is a single WASM target that can be served statically (including from the existing Axum binary).

Cost accepted: every interaction crosses HTTP and the WASM bundle is downloaded on first load. Acceptable for an internal tool. Heavy transfers can still use streaming / SSE / WebSocket if ever needed.

## Constraints to Honor Now

These keep the later Leptos rewrite frontend-only:

- Keep `db` / `sftp` / `io` logic behind real JSON REST endpoints.
- Keep HTMX-specific conventions (`HX-*` headers, fragment swapping, partial-HTML responses) in the template/handler layer — never in the service layer / core API logic.
- Do not rely on Askama templates calling services in-process and bypassing the REST boundary. The frontend should consume the API contract.

## Alternatives Considered

- **Dioxus** — React-like virtual DOM, built on Axum, cross-platform (web/desktop/mobile/TUI). Viable, but Leptos was chosen for the web-first rewrite.
- **Topcoat (tokio-rs)** — battery-included, server-rendered framework; renders HTML server-side and translates small Rust expressions to JavaScript for client interactivity (no WASM). Marked "early-stage and experimental — expect breaking changes." Not chosen; revisit later.
- **Staying on HTMX indefinitely** — valid; reconsider if the interactive requirements outgrow it.

## References

- Leptos: <https://github.com/leptos-rs/leptos> — full-stack framework, `leptos_axum` integration, `start-axum` template.
- Dioxus: <https://dioxuslabs.com/> — fullstack built on Axum.
- Topcoat: <https://github.com/tokio-rs/topcoat> — tokio-rs, experimental.
- HTMX: <https://htmx.org/>
- Askama: <https://github.com/askama-rs/askama> — Rust HTML template engine.