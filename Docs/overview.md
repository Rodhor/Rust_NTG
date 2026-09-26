# Docs Overview

> **Purpose:** Track forward-looking ideas and decisions. Items in this folder are
> **not currently relevant** — they are recorded for reconsideration at a later date.
> Full information lives in the linked docs; summaries live here.

## Index

| Doc | Topic | Summary |
| --- | --- | --- |
| [fullstack-rust.md](fullstack-rust.md) | Full Rust stack — frontend path (deferred) | Goal: a fully Rust web app. Now (decided 2026-09-24): **API-only**, no frontend — pure Axum JSON REST API, tested via Bruno. Later: add a frontend that consumes the API, most likely **Leptos as an SPA (CSR)**; backend stays untouched. Chosen over isomorphic Leptos, HTMX, Dioxus, and Topcoat. The REST API is the contract that makes a later frontend purely additive. |
| [sftp-auth-key.md](sftp-auth-key.md) | SFTP key-based auth (deferred) | Now (decided 2026-09-26): **password auth**, host key pinned. The SFTP server is outside our network, so it is a password-guessing surface and the credential is a reusable plaintext secret. Later: ed25519 key via `ssh-keygen` (setup step, not app code) + server-side `authorized_keys` restrictions, then `PasswordAuthentication no`. Password is kept as the fallback for password-only test servers. |
| [api-bind-address.md](api-bind-address.md) | API exposure and credential delegation | Now (decided 2026-09-26): `127.0.0.1` only — loopback is the entire access-control story, so no API auth or TLS is needed. **Open contradiction:** a frontend on a different internal server cannot reach a loopback-bound API; if the bind is ever widened, the API becomes a credentialed proxy onto an internet-facing SFTP server and auth becomes mandatory in the same change. |

## Status Legend

- **Planned** — recorded intent, not yet acted on; revisit later.
- **Reconsider** — option previously considered and set aside; re-evaluate if requirements change.