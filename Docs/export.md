# Export — ERP CSV and Picture CSV

> **Status: Decided 2026-09-26. In force, but the transport is now contested.**
> One items CSV plus one picture CSV per export run. Selection is driven by
> `last_exported_hash` so unchanged items are not re-sent.
>
> **Updated 2026-09-27.** The *selection logic, hashing and bookkeeping in this
> document are unaffected* by the deployment change, and the endpoints still work.
> What is now open is **whether CSV is the delivery mechanism at all** — the
> Linux server can reach the ERP's GraphQL endpoints, so a direct push is
> available and would remove the positional-import risk described below. See
> [open-decisions.md](open-decisions.md) items 13 and 14. Everything here that is
> about *choosing what to send* survives; everything about *the file format* is
> contingent on that decision.

## Selection

Only two things put an item into an export:

1. it is **`approved`**, and
2. its `content_hash` differs from `last_exported_hash` (or was never exported).

```sql
SELECT * FROM items
 WHERE status = 'approved'
   AND (last_exported_hash IS NULL OR last_exported_hash <> content_hash)
 ORDER BY first_seen_at, id;
```

`last_exported_hash` is what makes "only re-send if it changed" work. Without it
every export re-sends every approved item and the ERP upserts all of them for no
reason.

**Updates and new items go in the same CSV.** The ERP upserts on the key column,
so it distinguishes them. That is fine for the CSV, and it is why the future
GraphQL path needs the distinction moved into this program — see
[open-decisions.md](open-decisions.md).

## Flow

```
POST /api/v1/exports
  │
  ├─ 1. select the export set (above)
  ├─ 2. write items CSV  -> EXPORTS_DIR/<timestamp>-items.csv
  ├─ 3. for items whose picture is not present locally:
  │        download from SFTP into PICTURES_DIR
  │        record picture_path + picture_state
  ├─ 4. write picture CSV -> EXPORTS_DIR/<timestamp>-pictures.csv
  ├─ 5. UPDATE items SET last_exported_hash = content_hash  (only the exported set)
  ├─ 6. record the `exports` row: rev, counts, paths, failed_pictures
  │
  └─ 7. return { item_count, csv_path, pictures_csv_path, failed_pictures[] }
```

Step 5 must be scoped to the selected set. An unconditional
`UPDATE items SET last_exported_hash = content_hash` would silently mark
unexported items as sent.

## The Items CSV

- **Header text comes from `ErpField::header()`** — never from the database. See
  [field-mapping.md](field-mapping.md).
- Only **active** slots are written, in slot order (`ORDER BY id`).
- Values are the output of `project(blob, field_config)` — recomputed at export
  time, not stored at import time.
- The key column (EAN or SKU) is one of the active slots, so it is present in the
  file. The ERP's upsert key must be a column in the CSV.
- The item UUID is **never** written. The ERP has no use for it.

Streamed with `csv::Writer` — a 100 000-row × N-field export must not be built in
memory. Same reason as the import side.

### The positional-import risk

The ERP **reads by position**. Identical headers exist so a human can debug the
file; they are not the matching mechanism. This is fully described in
[field-mapping.md](field-mapping.md) and is **unresolved**:

- if inactive slots are omitted and the row shifts left, a dropped slot in the
  *middle* moves every later field into the wrong slot, and the import **succeeds
  with wrong data**.

Until it is decided, `write.rs` must not assume either behaviour. This is the
highest-severity open item in the project.

## The Picture CSV

Separate file, containing picture path and name only. The ERP expects a
filepath; nothing more is required, and **pictures are not displayed in the UI**.

Ordering, if the ERP needs it: the item CSV first, the picture CSV second, since
the picture rows reference items by key.

### Failed picture downloads do not block the export

Confirmed: the item still goes into the ERP CSV, with whatever picture path state
exists. The export response and the export log record `failed_pictures` so the
failures are visible and retryable, and the item's `picture_state` is set to
`failed` so a later run can pick it up.

This is the right trade because the ERP item data is the valuable part; a missing
photo should not block selling.

## Picture Storage

- `PICTURES_DIR/<item identity>/<filename>`, or a flat directory keyed by the
  item's UUID.
- **Filename source is undecided** — see [open-decisions.md](open-decisions.md).
  Using the item's UUID as the directory or file stem dedupes naturally and keeps
  arbitrary characters from the feed out of the filesystem; the original filename
  is more readable but collides and must be sanitised.
- Download only for **approved** items, not at import. The user only wants
  pictures for what they chose to sell.
- Existence is checked before downloading, so re-runs are cheap and a re-export
  does not re-fetch.

## Re-Export Everything

```
POST /api/v1/exports { "all": true }
```

Implemented as `UPDATE items SET last_exported_hash = NULL` scoped to approved
items, then the normal flow. Needed when the ERP import is believed to have failed
or partially applied.

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/exports` | Generate an export; `{ "all": true }` to re-export all approved |
| `GET` | `/api/v1/exports` | Export log, paginated, newest first |
| `GET` | `/api/v1/exports/{id}/items.csv` | Download the items CSV |
| `GET` | `/api/v1/exports/{id}/pictures.csv` | Download the picture CSV |
| `POST` | `/api/v1/exports/{id}/retry-pictures` | Re-attempt only the failed picture downloads |

CSV files are served by streaming from disk (`tokio::fs::File` + `ReaderStream`),
not by loading them into memory.

## Related

- [data-model.md](data-model.md) — the two hashes and the export predicate.
- [field-mapping.md](field-mapping.md) — the projection and the positional risk.
- [import.md](import.md) — where `content_hash` comes from.
