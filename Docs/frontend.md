# Frontend Choice

> **Status: Decided 2026-09-26, revised 2026-09-27. In force when the UI is
> built.**
> Server-rendered HTML from the existing Axum binary. The UI is not on the
> critical path for the API phases.
>
> **Revised 2026-09-27:** the app now runs on a Linux server and the operator's
> browser is on Windows, so the app can no longer open a browser — it logs its
> URL instead. It remains same-origin behind nginx, which is all the CORS
> reasoning below depended on.

## The Decision

**Server-rendered HTML.** `askama` templates mounted into the existing Axum
router. The shipped artefact is the same single binary; on startup it binds
`127.0.0.1:8080` and logs the URL. nginx puts it on HTTPS for the browser.

No WASM. No JavaScript framework. No second build. No second process.

Explicitly confirmed as acceptable by the user: **a browser window is fine, a
native window is not required.**

## The Input That Decided It: The Actual UI Inventory

A frontend choice should be judged against the screens that will exist, not
against a vague sense of "the UI might grow". These are the six:

| # | Screen | Shape |
| --- | --- | --- |
| 1 | **Import log** | Table of ledger rows + an "Import now" button |
| 2 | **Item list** (the main screen) | Wide table, N display columns, per-row approve/reject, bulk select, filter by status/import, paginated |
| 3 | **Item detail** | Full field set for one item — a drawer or a separate page |
| 4 | **Field config** | One editable row per output slot: source, ops, order, active. Add/remove/reorder |
| 5 | **Export log** | Table + download links + list of failed picture downloads |
| 6 | **Settings** | SFTP config, folder paths, number locale |

Now the honest assessment: **five of these six are forms and tables.** The only
screen with genuine client-side state wants is #4 (reorderable op lists) and, to
a lesser degree, #2 (selection across pages). Neither of those requires a
reactive framework — they require a `<form>` and a checkbox array.

## Why Server-Rendered Wins Here

1. **Deployment is the stated priority, and this is the only option that
   achieves it with zero new build machinery.** One binary, one systemd unit, no
   second artefact to keep in sync. See [deployment.md](deployment.md).
2. **Bulk selection is *simpler*, not harder.** Screen 2's "select all on this
   page, then approve" is one `<form method="post">` with `id[]` checkboxes.
   In a SPA you hand-build selection state that HTML gives you for free.
3. **It is the same async runtime the backend already uses.** No second runtime,
   no IPC layer, no bridging tokio into a GUI thread. The SFTP and import work is
   directly reusable as-is.
4. **It escalates without a rewrite.** See the ladder below.
5. **The API is the real investment.** Every phase in [roadmap.md](roadmap.md)
   is backend work. A wrong frontend choice costs one folder; a wrong API shape
   costs everything. This decision is cheap to revisit, so it does not block
   Phases 1–8.

## Escalation Ladder

Server-rendered is the *first rung*, not a dead end. Each step is additive and
none of them require touching the backend:

1. **`askama` templates** — plain form POSTs, POST-redirect-GET.
2. **`askama` + `htmx` islands** — add `htmx` as one dependency and use it on
   individual pages (e.g. field-config op rows) for partial updates. Pages that
   don't need it stay plain.
3. **A SPA (Leptos/Dioxus CSR)** — only if 1 and 2 both fall short. The REST
   API is already the contract, so this swap is frontend-only.

Note this **reverses the earlier HTMX rejection** recorded in
[fullstack-rust.md](fullstack-rust.md), and for the same reason it was rejected
then: HTMX was set aside when there was no UI to augment. There will be a UI.
HTMX is now a *later increment on top of* templates, not an alternative to them.

## The Condition That Flips This Decision

There is exactly one honest argument for a WASM SPA, and it is not technical
efficiency:

> **If the goal is to learn a modern reactive Rust frontend, Leptos is the better
> teacher and the extra toolchain is the price.**

Leptos 0.8 is the current stable line and `leptos_axum` integrates with the
existing router. The WASM artefact builds identically regardless of host OS, so
CI gets slightly more complex but deployment stays easy. The costs are real: the
`wasm32` target, `trunk`/`cargo-leptos`, `wasm-opt` (there is a long-standing
issue about it hanging on install), and version coupling between `leptos` and
`leptos_axum`.

If that learning goal is the priority, switch to Leptos and change this
document. Otherwise stay on the ladder above.

## Why Not the Others

### Tauri

The correct Tauri architecture has the Rust backend as a **library** invoked
through Tauri commands, not an HTTP server. That was rejected because it
discards the REST contract the whole design converges on, and Bruno can no
longer exercise the app.

The alternative — Tauri opening a webview at `http://localhost:8080` — is a
native window wrapped around a browser tab. It buys "no typing the URL" and
costs a Rust + TypeScript + bundler toolchain, a distributed-app upgrade path,
and an IPC layer. Cosmetic benefit, real cost.

Revisit only if a native window becomes a requirement.

### iced / egui / Slint (native Rust GUI)

- **No usable data grid.** `iced::widget::table` exists but is bare — you build
  sorting, selection and pagination yourself. `egui` needs a third-party crate
  (`egui_table` / `egui-data-table`) for a table of this shape.
- **Async friction against existing code.** The backend is tokio with a blocking
  SFTP layer. A native GUI needs a channel bridge and careful thread handoff.
- **Routing.** A native app that later hosts *several* tools has to reinvent
  navigation, and does it worse than HTTP routes.
- **Churn.** egui's own README is direct about it: *"If you want a GUI to break
  when you upgrade, egui isn't for you. Yet."*

Slint additionally has licensing that needs checking before commercial use.

### One `index.html` + `fetch`, no framework

Genuinely the least code and an honest option. Rejected because it takes the UI
out of Rust for a project whose stated purpose includes learning Rust, and you
would hand-write DOM code for the grid anyway. This is the fallback if avoiding
dependencies ever matters more than language consistency.

## Implementation Notes for Whenever This Gets Built

- `askama` compiles templates into type-checked Rust; a missing field is a
  compile error, not a blank page. `askama_web` with the `axum-0.8` feature
  provides the `IntoResponse` impl.
- Keep the JSON API and the HTML routes separate concerns: HTML routes under
  `/` (or `/ui`), JSON under `/api/v1`. They share handlers and services; only
  the response type differs. This is what keeps the frontend replaceable.
- Static assets (one small CSS file, maybe one JS file) served with
  `tower_http::services::ServeDir`. No bundler, no npm.
- Escaping is on by default in `askama`; never reach for `|safe` on data that
  originates from the imported CSV. That data is untrusted input.
- Opening the browser on startup: **no longer applicable.** The app runs on a
  Linux server and the browser is on a different machine, so it logs the URL it
  is serving instead. The nicety it bought — "no typing the URL" — is now
  provided by a bookmark.
- **No journal-mode configuration is needed for the UI to stay responsive.** The
  previous version of this document required SQLite in **WAL** mode, because
  otherwise a long import held the write lock and the UI's reads blocked,
  freezing the page mid-import. On PostgreSQL, MVCC gives readers a consistent
  snapshot while a writer works, so the problem does not exist and there is
  nothing to configure.

## Related

- [api-surface.md](api-surface.md) — the contract the UI consumes.
- [deployment.md](deployment.md) — why one binary is the goal.
- [fullstack-rust.md](fullstack-rust.md) — the superseded earlier decision.
