# Full Rust Stack — Frontend Path

> **Status: Forward-looking. Not currently relevant — revisit at a later date.**
> This doc records an intended future direction, not something to act on today.

## The Goal

A web application built entirely in Rust:

- **Backend:** Rust (Axum) — a JSON REST API. This is the **current** focus.
- **Frontend (now):** none. The API is exercised directly with a tool such as Bruno.
- **Frontend (later):** a frontend consuming the REST API, most likely **Leptos** as a single-page application (client-side rendering / WASM). Purely additive — no backend changes required.

## Decision (2026-09-24): API-only scope

For now, this project builds a pure JSON REST API with **no frontend**. Reasoning:

- A UI is not needed to validate the API; Bruno exercises the same endpoints faster than building a frontend would.
- YAGNI: don't build a frontend until someone actually uses one.
- The API contract is the thing worth getting right first. A frontend is then cheap to add later — the API must be stay-frontend-agnostic.

## Intended Path

1. **Phase 1 — now:** Axum JSON REST API only. No HTML rendering, no templates, no HTMX anywhere in the request path.
2. **Phase 2 — later:** Choose and add a frontend that talks to the existing REST API (see Leptos below). Frontend-only work, done client-side or as a purely additive static/served layer.

## Leptos Mode Decision: SPA (CSR), not Isomorphic

When the frontend is eventually built, the plan is a Leptos SPA rather than isomorphic (SSR + hydration):

- **SEO / crawlability** — the largest win of isomorphic SSR. Irrelevant for an internal tool.
- **First-paint performance** — matters on public, content-heavy sites. Irrelevant on an internal network.
- **Server functions (`#[server]`)** — the main selling point of isomorphic Leptos, but they duplicate what the REST API already does. Choosing isomorphic would mean either redundant server functions or Leptos pages calling the REST API anyway — i.e. an SPA with extra build complexity.
- **Build complexity** — isomorphic requires `cargo-leptos`, two compile targets (server + WASM hydrate), and hydration plumbing. An SPA is a single WASM target that can be served statically (including from the existing Axum binary).

Cost accepted: every interaction crosses HTTP and the WASM bundle is downloaded on first load. Acceptable for an internal tool. Heavy transfers can still use streaming / SSE / WebSocket if ever needed.

## Constraints to Honor Now

These are requirements of the current API design, not just hedges for the eventual frontend:

- All `db` / `sftp` / `io` logic lives behind real JSON REST endpoints. The API is the contract.
- Return structured JSON, not partial HTML. No server-side rendering concerns leak into the service layer.
- Keep HTTP semantics clean (status codes, content types, error bodies) so any client — Bruno today, a browser SPA later — works without backend changes.

## Alternatives Considered

- **HTMX** — server-rendered HTML with hypermedia-driven interactivity. Viable for a simple frontend, but rejected for now: there is no UI, so there is nothing for HTMX to augment. Revisit only if a lightweight server-rendered frontend is wanted later.
- **Dioxus** — React-like virtual DOM, built on Axum, cross-platform (web/desktop/mobile/TUI). Viable, but Leptos was chosen for the web-first rewrite.
- **Topcoat (tokio-rs)** — battery-included, server-rendered framework; renders HTML server-side and translates small Rust expressions to JavaScript for client interactivity (no WASM). Marked "early-stage and experimental — expect breaking changes." Not chosen; revisit later.
- **Staying API-only indefinitely** — valid; reconsider if a UI is ever genuinely needed.

## References

- Leptos: <https://github.com/leptos-rs/leptos> — full-stack framework, `leptos_axum` integration, `start-axum` template.
- Dioxus: <https://dioxuslabs.com/> — fullstack built on Axum.
- Topcoat: <https://github.com/tokio-rs/topcoat> — tokio-rs, experimental.
- HTMX: <https://htmx.org/>