# Docs Overview

> **Purpose:** Track forward-looking ideas and decisions. Items in this folder are
> **not currently relevant** — they are recorded for reconsideration at a later date.
> Full information lives in the linked docs; summaries live here.

## Index

| Doc | Topic | Summary |
| --- | --- | --- |
| [fullstack-rust.md](fullstack-rust.md) | Full Rust stack — frontend migration path | Goal: a fully Rust web app. Now: Axum + HTMX + Askama (server-rendered). Later: rewrite the frontend in **Leptos as an SPA (CSR)**; backend (Axum REST API + `db`/`sftp`/`io`) stays untouched. Chosen over isomorphic Leptos and over Dioxus / Topcoat. REST API is the contract that makes the rewrite frontend-only. |

## Status Legend

- **Planned** — recorded intent, not yet acted on; revisit later.
- **Reconsider** — option previously considered and set aside; re-evaluate if requirements change.