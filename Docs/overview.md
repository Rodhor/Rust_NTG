# Docs Overview

> **Purpose:** Track forward-looking ideas and decisions. Items in this folder are
> **not currently relevant** — they are recorded for reconsideration at a later date.
> Full information lives in the linked docs; summaries live here.

## Index

| Doc | Topic | Summary |
| --- | --- | --- |
| [fullstack-rust.md](fullstack-rust.md) | Full Rust stack — frontend path (deferred) | Goal: a fully Rust web app. Now (decided 2026-09-24): **API-only**, no frontend — pure Axum JSON REST API, tested via Bruno. Later: add a frontend that consumes the API, most likely **Leptos as an SPA (CSR)**; backend stays untouched. Chosen over isomorphic Leptos, HTMX, Dioxus, and Topcoat. The REST API is the contract that makes a later frontend purely additive. |

## Status Legend

- **Planned** — recorded intent, not yet acted on; revisit later.
- **Reconsider** — option previously considered and set aside; re-evaluate if requirements change.