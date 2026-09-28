# Full Rust Stack — Frontend Path (Superseded)

> **Status: Superseded 2026-09-26 by [frontend.md](frontend.md).**
> Kept because the reasoning is worth understanding, and because the HTMX
> re-reversal below is a useful lesson. **Do not follow this document.**

## What This Document Decided

On 2026-09-24 the project was scoped as **API-only**: a pure Axum JSON REST API,
no frontend, exercised with Bruno. The frontend question was deferred on two
grounds:

- a UI is not needed to validate the API, and Bruno exercises the same endpoints
  faster than building a frontend would,
- YAGNI — don't build a frontend until someone actually uses one.

The chosen future path was **Leptos as a client-side-rendered SPA**, purely
additive, with the backend untouched. HTMX, Dioxus and Topcoat were considered
and set aside.

## Why It Was Superseded

The premise changed, not the reasoning. A UI became a **confirmed requirement**
(a single user deciding which items to sell, in front of 400 source columns), and
the user added a second consideration: the UI may need to host **several tools**
over time, with the CSV tool being only the first.

That second point is the one that actually changed the answer. A suite of tools
wants a shared shell, client-side routing and no full page load when moving
between tools — which is a genuine argument for a SPA and a weak one against
server-rendered templates.

## What the Current Decision Is

See [frontend.md](frontend.md). In short: **server-rendered HTML via `askama`
in the existing Axum binary**, with the `.exe` opening the browser.

The deciding input was the actual screen inventory — five of the six screens are
forms and tables — combined with the standing preference for a single copied
binary as the deployment artefact. A WASM SPA would add a second build system
and a version lock to save effort on screens that do not need it.

The escalation path is recorded rather than dismissed: `askama` → `htmx` islands
on the pages that need them → a SPA only if both fall short. The REST API is
already the contract, so every step of that ladder is frontend-only.

## The HTMX Reversal, and Why It Is Not a Contradiction

HTMX was rejected here **for the same reason it is now back on the ladder**:
there was no UI to augment. Server-rendered HTML plus hypermedia-driven
interactivity needs an existing interface to drive.

This is not a reversal of judgement, it is the same judgement applied at a
different time. The general lesson worth keeping: *deferring a decision is
correct; freezing the reasoning behind it is not.* Record what was true when the
decision was made, and revisit it when that changes.

## Constraints That Survived Into The Current Decision

These were the right call then and remain right now — the API is the contract,
and it is what makes the frontend replaceable:

- all `db` / `sftp` / `io` logic lives behind real endpoints,
- structured JSON, not partial HTML, from the service layer,
- clean HTTP semantics so any client works without backend changes,
- **and the one this project most needs now:** versioned, namespaced resources, so
  a second tool is `/api/v1/<tool>/…` rather than a redesign.

## References

- Leptos: <https://github.com/leptos-rs/leptos> · 0.8 is the current line.
- Dioxus: <https://dioxuslabs.com/>
- Topcoat (tokio-rs): <https://github.com/tokio-rs/topcoat> — experimental.
- HTMX: <https://htmx.org/>
- Askama: <https://github.com/askama-rs/askama> — with `askama_web` for the
  axum 0.8 integration.
