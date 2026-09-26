# Import — SFTP Fetch and Ingest

> **Status: Decided 2026-09-26. In force.**
> The import is the *difference* between what the remote directory holds and what
> the ledger records. It never assumes "today's file".

## Definition: Catch-Up, Not Schedule

The app is on-demand (see [deployment.md](deployment.md)), so it is routinely not
running when files arrive. The import is therefore **not** "fetch the file for
today". It is:

```
to import  =  what the remote directory contains
           −  what the imports ledger already records
```

If the machine was off for three days, three days of files are imported. If it
was off for a month and the remote directory only keeps a week, only that week is
imported — and that is correct behaviour, not data loss by this app.

This makes **idempotency mandatory**. The ledger's `UNIQUE (remote_path, size)`
is the mechanism: a given remote file at a given size is imported exactly once,
ever.

## Flow

```
POST /api/v1/imports
  │
  ├─ 1. open one SFTP session (see note on the pool below)
  ├─ 2. list SFTP_DEFAULT_IN_DIRECTORY
  ├─ 3. diff against `imports` on (remote_path, size)  →  the work list
  ├─ 4. for each file, in one transaction:
  │     a. download to IMPORTS_DIR/<name>            (local copy = re-derivable)
  │     b. read the header row
  │     c. upsert newly discovered columns into `source_columns`
  │        mark columns not seen this time → candidate deprecation
  │     d. stream rows:
  │          - project(row) -> values in slot order
  │          - content_hash(values)
  │          - match existing item by natural key (EAN, then SKU)
  │            · match + same hash   → unchanged
  │            · match + new hash    → update blob, content_hash, last_import_id
  │            · no match            → insert with a fresh UUID
  │     e. record counts on the `imports` row, status = 'ok'
  │
  └─ 5. return per-file summary { inserted, updated, unchanged, failed }
```

Any error in step 4 aborts that file's transaction and marks the ledger row
`status = 'failed'` with the message. Everything else in the work list continues.

## One Transaction Per File

A file's rows are upserted inside a single transaction. A duplicate key or a
malformed row therefore leaves **no half-state**: the ledger stays truthful and
re-running is safe. This is why the "duplicate key is an error" decision costs
nothing — it fails one file, not the day's import.

The error should name **file, row number and key**, because the user's job when
this fires is to find that row in the source and understand why.

There is deliberately **no quarantine table**. Duplicate keys "should not happen
by law"; the first time one actually does is the time to build the escape hatch.

## Natural-Key Matching

Full rule in [data-model.md](data-model.md). In short: EAN first, then SKU; both
resolving to *different* rows is an error; neither means insert.

A row with **neither** EAN nor SKU is currently treated as an error. Flagged in
[open-decisions.md](open-decisions.md) — the alternative is falling back to
`"row-<n>"` as a synthetic key, which is stable within a file but not across
re-imports of a changed file.

## Streaming and Memory

400 columns × up to 100 000 rows will not fit in memory if materialised
(~800 MB as `Vec<Vec<String>>`). Therefore:

- `csv::Reader` with a **reused `ByteRecord`**. Never collect a whole file.
- One row in flight at a time. Project → hash → upsert → next.
- `flexible(true)` on the reader; ragged rows must not abort the import.
- Delimiter, quote and "has header row" are configurable, because ERP and
  marketplace exports vary. Default `,` with a header row.

Project + hash is microseconds per row. The whole 100 000-row case is expected
at 1–3 seconds, dominated by upsert throughput. **Do not optimise before
measuring.**

## SFTP Session Handling

**Use one session for the whole import, not the pool.**

`src/sftp/pool.rs` implements a request-concurrency pool: a semaphore bounding
concurrent checkouts plus a free-list. That is the right shape for many
concurrent HTTP requests each needing a connection. It is the wrong shape for a
single once-a-day batch job that wants one long-lived connection.

For the import, the correct lifetime is: open once, use for the whole work list,
close at the end. The pool should not become a dependency of the import path.

**Known risk:** `Session::set_timeout` applies per operation. An import of
100 000 rows makes many operations on one session, so a total-workload timeout
can fire mid-import. The mitigation is a `keepalive` interval rather than a
generous timeout, and treating a timeout as "reconnect and continue from the
last committed row" rather than as a file-level failure. Confirm during
implementation.

## What the App Does To Source Files On The Server

**Undecided.** The config already has `SFTP_DEFAULT_OUT_DIRECTORY` and
`SFTP_DEFAULT_ARCHIVE_DIRECTORY`, but whether the app moves, renames or deletes
consumed files remotely is not. Recorded in [open-decisions.md](open-decisions.md).

The interaction to watch for: if files are moved or deleted remotely, the
`imports` ledger keyed on `remote_path` can no longer see them, and the
catch-up diff becomes meaningless for anything already removed. The local copy
in `IMPORTS_DIR` is what makes re-derivation possible in that case.

## Column Discovery and Deprecation

On every import the header row is read and reconciled with `source_columns`:

- **New name** → insert a row, `state = 'active'`, `first_seen = now`.
- **Existing name** → update `last_seen`.
- **Seen before, absent now** → set `state = 'deprecated'`. Deprecation is a
  *signal*, not an action: nothing is deleted, and the user decides what
  deprecation means in the UI.
- **Rename** (count matches, one name new, one gone) → the old row becomes
  `state = 'renamed'` with `renamed_to` set, and a new row is inserted. Rename
  support is required, not optional, because rules resolve by name and a rename
  would otherwise silently break every rule pointing at the old name.

## Maintenance: Re-Evaluation

Because import only re-hashes rows present in the incoming file, a field-config
change leaves items absent from recent files stale. The explicit
`POST /api/v1/maintenance/re-evaluate` action walks all items and, in one pass:

1. recomputes `content_hash` from the stored blob against the current config,
2. refills any newly added display columns,
3. backfills `hashed_at_rev`.

This is the same machinery as the display-column migration backfill — one
endpoint, not two. It is expected to be rare (a legal requirement changing what
must be tracked is the motivating case).

## Endpoints

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/imports` | Run an import; returns the per-file summary |
| `GET` | `/api/v1/imports` | The ledger, paginated, newest first |
| `GET` | `/api/v1/imports/{id}` | One ledger row with its error, if any |
| `GET` | `/api/v1/source-columns` | Discovered schema, with deprecation state |
| `POST` | `/api/v1/maintenance/re-evaluate` | Rehash + backfill all items |

## Related

- [data-model.md](data-model.md) — tables and the natural-key rule.
- [field-mapping.md](field-mapping.md) — what `project()` actually does.
- [export.md](export.md) — the other half of the loop.
