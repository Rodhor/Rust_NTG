# Data Model

> **Status: Decided 2026-09-26. In force.**
> SQLite, five tables, UUID primary keys on items, one JSON blob per item as the
> archive of record, real columns only for what is queried.

## Principles

1. **Operational state is columns. Data is a blob.** Columns exist for things
   that are filtered, sorted, joined or compared. Everything the source file
   contained lives in the blob.
2. **The blob is the archive; the columns are the index.** Both exist on
   purpose. The blob is what lets a *newly added* field be derived later without
   re-fetching from the SFTP server — which matters because source files are
   transient and may be gone from the remote directory by then.
3. **The cleaned projection is never stored.** It is recomputed from the blob at
   export time. That way changing the field config only requires a re-export, not
   a re-import. See [field-mapping.md](field-mapping.md).
4. **Raw data is strings.** No type coercion at ingest, because the rules decide
   what a column means. Numeric and date parsing happens in one place, at
   evaluation time.

## Schema

`src/db/migrations/0001_init.sql`:

```sql
-- ── Ledger of every file ever fetched ──────────────────────────────────────
CREATE TABLE imports (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    remote_path  TEXT    NOT NULL,
    filename     TEXT    NOT NULL,
    size         INTEGER NOT NULL,
    fetched_at   TEXT    NOT NULL,
    row_count    INTEGER,
    inserted     INTEGER NOT NULL DEFAULT 0,
    updated      INTEGER NOT NULL DEFAULT 0,
    unchanged    INTEGER NOT NULL DEFAULT 0,
    status       TEXT    NOT NULL CHECK (status IN ('ok', 'failed')),
    error        TEXT,
    -- Idempotency: a remote file of a given size is imported exactly once.
    UNIQUE (remote_path, size)
);

-- ── Auto-discovered source schema ──────────────────────────────────────────
CREATE TABLE source_columns (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    ntg          TEXT UNIQUE,                 -- NULL until mapped to a slot
    source_name  TEXT    NOT NULL,
    source_index INTEGER NOT NULL,
    first_seen   TEXT    NOT NULL,
    last_seen    TEXT    NOT NULL,
    state        TEXT    NOT NULL CHECK (state IN ('active', 'deprecated', 'renamed')),
    renamed_to   TEXT
);
CREATE UNIQUE INDEX source_columns_name_idx ON source_columns (source_name);

-- ── Items: one row per source item ─────────────────────────────────────────
CREATE TABLE items (
    id                 TEXT    PRIMARY KEY,          -- uuid v4
    ean                TEXT,
    sku                TEXT,
    blob               TEXT    NOT NULL,             -- JSON, empty values stripped
    content_hash       BLOB    NOT NULL,             -- 16 bytes, over the projection
    hashed_at_rev      INTEGER NOT NULL DEFAULT 0,   -- profile rev the hash was made at
    last_exported_hash BLOB,                          -- NULL until first exported
    status             TEXT    NOT NULL DEFAULT 'undecided'
                               CHECK (status IN ('undecided', 'approved', 'rejected')),
    decided_at         TEXT,
    picture_path       TEXT,
    picture_state      TEXT    CHECK (picture_state IN ('pending', 'ok', 'failed')),
    first_import_id    INTEGER NOT NULL REFERENCES imports(id),
    last_import_id     INTEGER NOT NULL REFERENCES imports(id),
    first_seen_at      TEXT    NOT NULL,
    updated_at         TEXT    NOT NULL
);

-- A surrogate PK means natural-key uniqueness is no longer enforced by the
-- schema, so enforce it explicitly. Partial indexes because a NULL may repeat.
CREATE UNIQUE INDEX items_ean_uq ON items (ean) WHERE ean IS NOT NULL;
CREATE UNIQUE INDEX items_sku_uq ON items (sku) WHERE sku IS NOT NULL;

-- Supports the item list's default query and the export selection.
CREATE INDEX items_status_idx ON items (status, updated_at);

-- ── User-configured output slots ───────────────────────────────────────────
CREATE TABLE field_config (
    id     INTEGER PRIMARY KEY,               -- slot number = output order
    ntg    TEXT    NOT NULL,                  -- must parse to ErpField
    source TEXT    NOT NULL,                  -- JSON-serialised Source
    ops    TEXT    NOT NULL,                  -- JSON-serialised Vec<Op>
    active INTEGER NOT NULL DEFAULT 1
);

-- Single-row revision counter for the field config as a whole.
CREATE TABLE profile (
    id  INTEGER PRIMARY KEY CHECK (id = 1),
    rev INTEGER NOT NULL
);
INSERT INTO profile (id, rev) VALUES (1, 1);

-- ── One row per generated CSV ──────────────────────────────────────────────
CREATE TABLE exports (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at        TEXT    NOT NULL,
    rev               INTEGER NOT NULL,      -- profile rev used
    item_count        INTEGER NOT NULL,
    csv_path          TEXT    NOT NULL,
    pictures_csv_path TEXT,
    failed_pictures   INTEGER NOT NULL DEFAULT 0
);
```

## Why `items.id` Is a UUID

The item's natural key is the EAN (probably) or the SKU, and it is not yet
decided which. A natural key as primary key would mean:

- choosing wrong today means migrating the primary key later,
- rows where the key is *absent* cannot be stored at all.

A surrogate `uuid` primary key removes both problems. The cost is that the
natural key is no longer enforced by the schema — which is why the two partial
unique indexes above exist. They turn a matching-logic bug into a constraint
violation instead of a silent duplicate.

**Both `ean` and `sku` are stored as columns** regardless of which is the
identity. That is deliberate cheap insurance: switching the identity later is a
migration, not a backfill over the blob.

## Natural-Key Matching During Import

With a surrogate key, an incoming row has no `id`, so the "same item?" question
must be answered before inserting. The rule:

1. Look up by `ean` if present.
2. Otherwise look up by `sku` if present.
3. If **both** resolve to *different* rows → error. Two existing items now claim
   the same item; a human has to look.
4. If neither resolves → insert a new row with a fresh UUID.

**The case that will actually bite:** a row whose EAN is unchanged but whose SKU
changed upstream. Matching on EAN (and overwriting the SKU) is correct;
missing would create a duplicate. Hence EAN first.

Consequence worth stating: `content_hash` is computed over the *projected*
values, not the raw row. So an incoming row that differs only in dropped columns
produces an unchanged hash, and neither the blob nor the export set moves. That
is the intended behaviour, and it is why there is only one hash rather than a
"source changed" hash and an "ERP output changed" hash.

## The Two Hashes

| Column | Computed over | Used for |
| --- | --- | --- |
| `content_hash` | `(ntg, value)` pairs of the active projection, in slot order | change detection between imports |
| `last_exported_hash` | the same value, copied at export time | selecting what still needs sending |

The export set is:

```sql
SELECT * FROM items
 WHERE status = 'approved'
   AND (last_exported_hash IS NULL OR last_exported_hash <> content_hash);
```

`last_exported_hash` is the column that makes "only re-send if it changed"
work. Without it every export re-sends every approved item, and the ERP
upserts all of them for no reason. "Re-export everything" is then just
`UPDATE items SET last_exported_hash = NULL`.

Algorithm: **SHA-256 truncated to 16 bytes** (`sha2`). Nothing fancier earns
its keep at this scale. Serialisation details are in
[field-mapping.md](field-mapping.md) — they are the difference between a correct
hash and one that produces phantom changes on a few percent of rows.

## Profile Revisions

`hashed_at_rev` records the `profile.rev` a hash was computed at. When the field
config is edited, `rev` is bumped and any item whose `hashed_at_rev` is stale is
reported as needing re-evaluation.

This matters because import only ever re-hashes rows that appear in the incoming
file. If the field config changes, items **absent from recent files** keep stale
hashes indefinitely. That is the "rehash all rows" case, and it needs an
explicit maintenance action rather than waiting for the next import. See
[import.md](import.md).

## Display Columns: Migration, Not JSON Extraction

The fields the user sees while deciding are stored as **real columns** on
`items`, not extracted from the blob at query time. The reason is that the UI
queries them; JSON extraction per row per page is the wrong shape for that.

The cost is explicit and accepted: **adding a display field is
`ALTER TABLE items ADD COLUMN` plus a backfill** of existing rows. So the
display set is treated as near-static, changed rarely, deliberately.

Note the deliberate separation:

- **Display fields** → real columns, changed by migration.
- **ERP output fields** → `field_config` rows only; values projected at export,
  never materialised.

Conflating these would force a migration on every ERP output change, which
would be wrong — ERP output changes are routine, display changes are not.

Both maintenance jobs — the migration backfill and the profile re-hash — are
the same pass over the table, and are served by one endpoint. See
[api-surface.md](api-surface.md).

## Storage and Performance

- Rows are sparse: ~150 of 400 source columns are dropped outright and ~80 more
  are usually empty, leaving roughly **170 populated keys ≈ 6 KB** per row.
- Worst case (100 000 rows) is therefore **~600 MB** of blob. SQLite handles this
  comfortably. Most days are 1–100 rows, so this is a number to *know*, not to
  optimise for.
- **Do not compress the blob.** It would make it unqueryable, and the plan is
  that it never needs to be queried.
- A full import at 100 000 rows is expected at **1–3 seconds**: project + hash is
  microseconds per row, and batched upserts in one transaction run at roughly
  50–100 k/s. No optimisation work is warranted before this is measured.
- **WAL mode must be set at startup** (`PRAGMA journal_mode=WAL`). Without it a
  long import holds the write lock and the UI's reads block, so the page appears
  frozen mid-import.

## Related

- [field-mapping.md](field-mapping.md) — what a hash is computed over.
- [import.md](import.md) — how rows get in.
- [export.md](export.md) — how rows get out.
- [open-decisions.md](open-decisions.md) — the unresolved parts of this schema.
