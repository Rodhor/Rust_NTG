# API Surface

> **Status: Decided 2026-09-26. In force for the JSON API.**
> Versioned REST under `/api/v1`, JSON only. HTML routes are a separate concern
> layered on top of the same handlers and services.

## Conventions

- Prefix: `/api/v1`. **Resources are namespaced so a second tool can be added
  later** (`/api/v1/erp/…`) without a redesign. The "multiple tools" concern is
  handled here, in the API, not in the frontend.
- Success and failure both use the existing `ApiResponse<T>` wrapper; errors use
  `ErrorBody` with the `AppError::status()` / `code()` mapping already in
  `src/domain/error.rs`. Every new `AppError` variant must be added there.
- Pagination is **keyset**, not `OFFSET`. With 100 000 rows whose state changes
  between page fetches, `OFFSET` skips and repeats rows. Keyset is
  `(updated_at, id) < (cursor_updated_at, cursor_id)`.
- Bulk endpoints take a **filter object, never a list of ids**. A bulk approve of
  10 000 items must be a set-based `UPDATE`, not 10 000 requests or a 400 KB
  JSON body.
- CSV dialect (delimiter, encoding, quoting, line endings) is whatever satisfies
  CSV rules; only the **headers** are fixed, and they come from `ErpField`.

## Endpoints

### Health

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/api/v1/health` | Exists today. Should report DB reachability once the DB layer lands. |

### Source schema

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/v1/source-columns` | Discovered schema with `state` (`active`/`deprecated`/`renamed`) |
| `POST` | `/api/v1/source-columns/{id}/deprecate` | User-driven deprecation |
| `POST` | `/api/v1/source-columns/{id}/restore` | Undo a deprecation |

Deprecation is a **signal, not a delete**. Nothing is ever removed from this
table; the user decides what deprecation means in the UI.

### Imports

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/imports` | Run an import. Returns per-file `{ filename, inserted, updated, unchanged }` or `{ filename, error }` |
| `GET` | `/api/v1/imports` | Ledger, keyset-paginated, newest first |
| `GET` | `/api/v1/imports/{id}` | One ledger row including `error` |

### Items

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/v1/items` | Item list. Params: `status`, `import_id`, `limit`, `cursor` |
| `GET` | `/api/v1/items/{id}` | One item, including the full blob for the detail view |
| `POST` | `/api/v1/items/decisions` | **Bulk.** Body: `{ filter, decision }` → returns affected count |

```jsonc
// POST /api/v1/items/decisions
{
  "filter": { "status": "undecided", "import_id": 42, "ids": ["…"] },
  "decision": "approved"          // approved | rejected
}
```

`ids` is optional inside the filter. Selection is set-based SQL in both cases.

`decision` is currently **one-way**: `undecided → approved | rejected` only, with
no way back. Confirmed as acceptable for now. Note the cost: a mis-click on
"approve all" is unrecoverable, and the fix is one enum variant plus one branch —
see [open-decisions.md](open-decisions.md).

### Field config

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/v1/fields` | All slots, in order, with the resolved source and ops |
| `PUT` | `/api/v1/fields` | Replace the whole config. Validates every `ntg` against `ErpField` and every op against its `Source` variant |
| `GET` | `/api/v1/fields/vocabulary` | The `ErpField` list with header text — what the UI offers |

`PUT` bumps `profile.rev` and the response reports **how many items went stale**,
so the UI can offer the re-evaluation action rather than silently going stale.
Replacing the whole config (rather than PATCHing one slot) is deliberate: the
config is a small ordered list that is edited as a unit, and a whole-document PUT
has no ordering or partial-failure semantics to get wrong.

### Exports

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/exports` | Generate an export. Body `{ "all": true }` to re-export every approved item |
| `GET` | `/api/v1/exports` | Export log, keyset-paginated, newest first |
| `GET` | `/api/v1/exports/{id}/items.csv` | Stream the items CSV from disk |
| `GET` | `/api/v1/exports/{id}/pictures.csv` | Stream the picture CSV from disk |
| `POST` | `/api/v1/exports/{id}/retry-pictures` | Re-attempt only failed picture downloads |

### Maintenance

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/maintenance/re-evaluate` | Rehash all items and backfill display columns. Expected to be rare |

### Future — out of scope

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/erp/sync` | GraphQL push to the ERP. **Not available yet** — the endpoint is local and requires auth, so this is not currently actionable |

This is the reason the item identity question is worth settling early: the CSV
path lets the **ERP** decide new-vs-update via upsert, but the GraphQL path
requires **this program** to decide. See [open-decisions.md](open-decisions.md).

## Error Variants to Add

`AppError` (`src/domain/error.rs`) needs, per `AGENTS.md`:

| Variant | Covers |
| --- | --- |
| `Db(sqlx::Error)` | Query, migration, constraint violation |
| `Csv` | Parse and write failures, ragged rows |
| `FieldConfig` | Unknown `ntg`, invalid op for a `Source` variant, unknown slot reference, self-reference |
| `Picture` | Download or write failure for one picture (non-fatal to the export) |
| `UnknownItem` | `GET /items/{id}` with an id that does not exist |

Each needs `#[from]` where a source chain exists, an `#[error(...)]` message, and
entries in `status()` and `code()`.

## HTML Routes Are Separate

Per [frontend.md](frontend.md), the UI is server-rendered and shares handlers and
services with the JSON API — only the response type differs. Concretely:

- `/api/v1/**` — JSON, `ApiResponse<T>`, used by Bruno and by the future SPA.
- `/` and `/items`, `/imports`, `/exports`, `/fields`, `/settings` — HTML via
  `askama`, using the same service functions.

If a second tool is ever added, it arrives as new resources under
`/api/v1/<tool>/` and new HTML routes under `/<tool>/`. Nothing existing moves.

## Related

- [frontend.md](frontend.md) — how the HTML layer consumes this.
- [data-model.md](data-model.md) — what the endpoints expose.
