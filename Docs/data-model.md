# Data Model

> **Status: Decided 2026-09-26, engine changed to PostgreSQL 2026-09-27. In force.**
> PostgreSQL, **six** tables, UUID primary keys on items, one JSON blob per item
> as the archive of record, real columns only for what is queried.
>
> **The schema below is the design, not yet the migration.** The actual
> `0001_init.sql` is being written now and is expected to supersede this document
> — see the "Status of the schema" note at the end. The *shape* (six tables,
> which columns, which indexes, why UUID) is decided; the exact DDL is not.

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
   evaluation time. This is about the *values inside the CSV*, not the column
   types: the operational columns are still strictly typed, and PostgreSQL
   enforces them. The rule means a German-formatted `1.234,56` is stored as the
   string the source actually contained, rather than guessed at and coerced into
   a number that is then wrong.

## Why PostgreSQL, Decided 2026-09-27

The engine started as SQLite and was changed to PostgreSQL before the first
migration was written. The reasoning, because the deciding factor is not the one
people expect.

**It is not about performance.** At 100 000 rows of ~6 KB, both engines are
comfortable and the import is seconds either way. Nobody would choose a database
engine on that basis for this workload.

**It is about what a committed migration costs.** A migration is the one artifact
that cannot be edited later without a new migration on top of it, so the engine
choice and the first migration are the same decision. Writing `0001_init.sql` in
SQLite would have permanently recorded `AUTOINCREMENT` and `TEXT` timestamps. The
code cost of switching later was measured at **31 lines in one file** — the
rewrite that was being avoided does not exist. The rewrite that *does* exist is
rewriting migration history.

**It is about the daily type tax.** This is the real reason, and it recurs on
every query for the life of the project:

| | SQLite | PostgreSQL |
| --- | --- | --- |
| Timestamp column | `TEXT` | `TIMESTAMPTZ` |
| What a query gives you | `String` | `DateTime<Utc>` / `OffsetDateTime` |
| What you write | parse it yourself, everywhere | nothing |

SQLite has no timestamp type at all, so a `created_at` column comes back as a
`String` from *every* query and gets hand-parsed in *every* query, forever. On
PostgreSQL it is a typed value for free. With compile-time macros the database is
what tells you the type, so this is a per-query cost, not a one-time one.

**It is about learning the right instinct.** SQLite is dynamically typed — it
uses column "affinity" and will quietly accept `'abc'` in an `INTEGER` column or
an integer in a `TEXT` column, converting silently. PostgreSQL will not. Since
this project is a learning exercise about data modelling, learning on a database
that enforces its own types is the point.

**What it costs, honestly:** a server to install, secure, back up and keep alive
instead of a file you can `cp`; a password in the connection URL, which is why
`DatabaseConfig` needs a redacting `Debug`; `pg_dump` instead of `cp`; and one
more thing that can be down when the app is up.

**The one thing that would have changed the answer:** if standing up PostgreSQL
locally were going to stall the work, SQLite-plus-31-lines was the right call for
now. The decision was made because the first migration is being written now, and
now is the cheapest moment it will ever be.

## Dialect: What Changes From the SQLite Draft

Recorded here so the dialect is a known, finite list rather than a surprise
discovered while writing the migration.

| SQLite draft | PostgreSQL | Note |
| --- | --- | --- |
| `INTEGER PRIMARY KEY AUTOINCREMENT` | `BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY` | Three tables: `imports`, `source_columns`, `exports`. See the footgun below. |
| `BLOB` | `BYTEA` | The two hash columns. `BYTEA` comparison with `<>` and `IS NULL` behaves as the export predicate needs. |
| `TEXT` timestamps | `TIMESTAMPTZ` | `fetched_at`, `first_seen`, `last_seen`, `decided_at`, `first_seen_at`, `updated_at`, `created_at`. |
| `PRAGMA journal_mode=WAL` | *deleted* | MVCC does this job. See "Storage and Performance". |
| `PRAGMA foreign_keys=ON` | *deleted* | Foreign keys are enforced by default in PostgreSQL, so the `REFERENCES` clauses in this schema are **actually live** — under SQLite they were inert unless the pragma was set. This is a correctness win, not just tidiness. |
| `INTEGER` | `INTEGER` (32-bit) | **Footgun:** SQLite's `INTEGER` is 64-bit; PostgreSQL's is 32-bit. The identity columns are `bigint`, so referencing columns must be `BIGINT` too — but see below, the mistake is **silent**. |

**The identity footgun, spelled out** because it is the kind of error that costs an
evening, and because it does *not* announce itself — see the section below.

**Everything else carries over unchanged.** Both engines support `CHECK`
constraints, `UNIQUE` constraints, and partial unique indexes with a `WHERE`
clause, so all five indexes in this schema are valid PostgreSQL as written:

- `source_columns_name_idx` — `UNIQUE` on `source_name`
- `items_ean_uq`, `items_sku_uq` — partial `UNIQUE ... WHERE col IS NOT NULL`
- `items_status_idx` — composite on `(status, updated_at)`
- `imports` — `UNIQUE (remote_path, size)` as a table constraint

### The `INTEGER` / `bigint` mismatch is silent, not loud

`GENERATED ALWAYS AS IDENTITY` defaults to **`bigint`** (the older `SERIAL` idiom
gives `integer`). The obvious mistake is therefore writing
`first_import_id INTEGER REFERENCES imports(id)` and expecting the same
misalignment to bite as it would with `AUTOINCREMENT`.

**It does not error.** Verified against PostgreSQL 18:

```sql
ALTER TABLE items ADD COLUMN bad_import_id INTEGER REFERENCES imports(id);
-- ALTER TABLE   ← accepted
```

PostgreSQL allows a foreign key between integer types of different widths, so
`int4 = int8` is a valid comparison and the constraint is created. Nothing
rejects it — not the DDL, and not sqlx, because the column genuinely exists as
`int4`. **The mismatch is therefore silent, which makes it more dangerous than a
loud failure**, and it is the reason this is written down at all. Use `BIGINT` for
the referencing columns, matching what they reference.

### Verified against a real server

The schema in this document was applied to **PostgreSQL 18** and the constraints
were exercised, rather than reasoned about. All of these were confirmed:

| Claim | Result |
| --- | --- |
| The six tables and four indexes apply cleanly | **Yes** |
| `AUTOINCREMENT` is rejected | **Yes** — syntax error |
| `BLOB` is rejected | **Yes** — `type "blob" does not exist` |
| `GENERATED ALWAYS AS IDENTITY` yields `bigint` | **Yes** |
| Foreign keys are enforced **with no pragma** | **Yes** — an insert with a bad `first_import_id` was rejected |
| The export predicate works on `BYTEA` | **Yes** — differing hashes → `true`, equal → `false`, `NULL` → `true` |
| Two rows with `NULL` `ean` are allowed | **Yes** — the partial index permits them |
| A duplicate `ean` is rejected | **Yes** — `duplicate key ... items_ean_uq` |
| `status` and `picture_state` `CHECK`s are enforced | **Yes** |
| `profile.id = 1` singleton `CHECK` is enforced | **Yes** |
| `UNIQUE (remote_path, size)` is enforced | **Yes** |

One behaviour worth knowing rather than being surprised by: **identity sequences
skip values.** A failed insert still consumes a value, so ids are not gapless —
observed directly, where a rejected insert left a hole. Treat these ids as opaque
and never assume `MAX(id) + 1`.

One behavioural difference worth knowing: PostgreSQL's default collation is
locale-dependent, so `ORDER BY` on a `TEXT` column can sort differently from
SQLite's byte-order default. This affects display order of `source_name` and
`first_seen_at` tie-breaks. If stable ordering ever matters, add an explicit
`COLLATE "C"` rather than relying on the server's locale.

## Schema

`src/db/migrations/0001_init.sql` — **as designed, dialect converted. The
migration itself is being written now and will supersede this block.**

```sql
-- ── Ledger of every file ever fetched ──────────────────────────────────────
CREATE TABLE imports (
    id           BIGINT  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    remote_path  TEXT    NOT NULL,
    filename     TEXT    NOT NULL,
    size         BIGINT  NOT NULL,
    fetched_at   TIMESTAMPTZ NOT NULL,
    row_count    BIGINT,
    inserted     BIGINT  NOT NULL DEFAULT 0,
    updated      BIGINT  NOT NULL DEFAULT 0,
    unchanged    BIGINT  NOT NULL DEFAULT 0,
    status       TEXT    NOT NULL CHECK (status IN ('ok', 'failed')),
    error        TEXT,
    -- Idempotency: a remote file of a given size is imported exactly once.
    UNIQUE (remote_path, size)
);

-- ── Auto-discovered source schema ──────────────────────────────────────────
CREATE TABLE source_columns (
    id           BIGINT  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    ntg          TEXT UNIQUE,                 -- NULL until mapped to a slot
    source_name  TEXT    NOT NULL,
    source_index INTEGER NOT NULL,
    first_seen   TIMESTAMPTZ NOT NULL,
    last_seen    TIMESTAMPTZ NOT NULL,
    state        TEXT    NOT NULL CHECK (state IN ('active', 'deprecated', 'renamed')),
    renamed_to   TEXT
);
CREATE UNIQUE INDEX source_columns_name_idx ON source_columns (source_name);

-- ── Items: one row per source item ─────────────────────────────────────────
CREATE TABLE items (
    id                 TEXT        PRIMARY KEY,          -- uuid v4
    ean                TEXT,
    sku                TEXT,
    blob               TEXT        NOT NULL,             -- JSON, empty values stripped
    content_hash       BYTEA       NOT NULL,             -- 16 bytes, over the projection
    hashed_at_rev      BIGINT      NOT NULL DEFAULT 0,   -- profile rev the hash was made at
    last_exported_hash BYTEA,                            -- NULL until first exported
    status             TEXT        NOT NULL DEFAULT 'undecided'
                                  CHECK (status IN ('undecided', 'approved', 'rejected')),
    decided_at         TIMESTAMPTZ,
    picture_path       TEXT,
    picture_state      TEXT        CHECK (picture_state IN ('pending', 'ok', 'failed')),
    first_import_id    BIGINT      NOT NULL REFERENCES imports(id),
    last_import_id     BIGINT      NOT NULL REFERENCES imports(id),
    first_seen_at      TIMESTAMPTZ NOT NULL,
    updated_at         TIMESTAMPTZ NOT NULL
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
    rev BIGINT NOT NULL
);
INSERT INTO profile (id, rev) VALUES (1, 1);

-- ── One row per generated CSV ──────────────────────────────────────────────
CREATE TABLE exports (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created_at        TIMESTAMPTZ NOT NULL,
    rev               BIGINT NOT NULL,      -- profile rev used
    item_count        BIGINT NOT NULL,
    csv_path          TEXT    NOT NULL,
    pictures_csv_path TEXT,
    failed_pictures   BIGINT NOT NULL DEFAULT 0
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

One thing got cheaper with the move to PostgreSQL: since version 11,
`ALTER TABLE ... ADD COLUMN` with a non-volatile default does **not** rewrite the
table, so the migration is near-instant and the backfill only needs to touch rows
where the new column is actually wrong. That lowers the cost of getting this
decision wrong, which is a reason to enumerate the display set deliberately
rather than to add columns speculatively and see what sticks.

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
- Worst case (100 000 rows) is therefore **~600 MB** of blob. PostgreSQL handles
  this comfortably. Most days are 1–100 rows, so this is a number to *know*, not
  to optimise for.
- **Do not compress the blob.** It would make it unqueryable, and the plan is
  that it never needs to be queried.
- A full import at 100 000 rows is expected at **1–3 seconds**: project + hash is
  microseconds per row, and batched upserts in one transaction run at roughly
  50–100 k/s. No optimisation work is warranted before this is measured.
- **No journal-mode configuration is needed.** The SQLite draft required
  `PRAGMA journal_mode=WAL` so a long import would not block the UI's reads.
  PostgreSQL's MVCC gives readers a consistent snapshot while a writer is
  working, so that problem does not exist and there is nothing to set at startup.
  This is a real simplification, not a cosmetic one: the previous version of this
  document carried a "must be set at startup, or the page appears frozen
  mid-import" warning that is now void.
- The nightly import and an interactive user can therefore overlap without the
  contention problem that would have needed `busy_timeout` tuning under SQLite.
  If they do overlap, PostgreSQL serialises the writers and both wait; neither
  gets an error.

## Compile-Time Queries and the `.sqlx` Cache

The database layer uses sqlx's **compile-time macros** (`query!` / `query_as!`),
so every SQL statement is checked against a real schema at build time. This is
the reason `0001_init.sql` must exist and be applied before the first query
compiles, and it is the reason the schema was fixed to an engine before the
migration was written.

The cost is that the build needs the schema available in one of two ways:

| Mode | How | When |
| --- | --- | --- |
| **Offline (default, what we use)** | `cargo sqlx prepare` writes query metadata to a checked-in `.sqlx/` directory, and `SQLX_OFFLINE=true` makes the build read it instead of a live database | CI, and any build where a database is not running |
| Online | `DATABASE_URL` points at a live, migrated database and the macros connect during compilation | Local iteration, and `cargo sqlx prepare` itself |

**`SQLX_OFFLINE=true` is not optional once any `query!` exists.** sqlx documents
that a present `DATABASE_URL` takes precedence over the offline cache, so a
leftover local database URL silently makes every build depend on a live database
and fail on a clean checkout with a confusing error rather than an obvious one.

**The gate, and what it actually catches.** `cargo sqlx prepare --check` is run
before a build, the same way tests are. Its coverage is asymmetric, which is worth
knowing so the gate is not over-trusted:

- A new query that has never been prepared → **fails**.
- A query whose result shape changed → **fails**.
- A **deleted** query whose metadata is still in `.sqlx` → **warning only**, and
  the build passes. Stale metadata accumulates quietly.
- With no `query!` macros at all, it **passes vacuously** — it cannot detect
  staleness about queries that do not exist yet.

`--check` compares metadata as JSON rather than as raw SQL, so comments and
whitespace churn in a query file is not a false alarm. It skips the
connectivity probe entirely when no `DATABASE_URL` resolves, which is correct for
offline mode.

## Status of the Schema

**The migration is being written now and is expected to supersede this document.**
The design is settled — six tables, which columns, which indexes, why a UUID
surrogate key, why the blob is the archive. What is not settled is the exact
DDL, and the following are known to be open when it is written:

- The **display column set** on `items` has not been enumerated. This has always
  been a migration-plus-backfill rather than JSON extraction; with PostgreSQL an
  alternative exists that SQLite made awkward, so it is worth revisiting
  deliberately rather than by default. See [open-decisions.md](open-decisions.md).
- Whether `blob` stays `TEXT` or becomes `JSONB`. `JSONB` validates and can be
  indexed; `TEXT` matches the current principle that the blob is never queried.
  The stated principle argues for `TEXT`, and `JSONB`'s main advantage —
  indexing — is explicitly not wanted.
- Timestamp handling in Rust: `Cargo.toml` currently enables sqlx's `chrono`
  feature while this document and the roadmap plan the `time` crate. Pick one
  before the migration hardcodes the expectation.

## Related

- [field-mapping.md](field-mapping.md) — what a hash is computed over.
- [import.md](import.md) — how rows get in.
- [export.md](export.md) — how rows get out.
- [deployment.md](deployment.md) — why the database is a server and not a file.
- [open-decisions.md](open-decisions.md) — the unresolved parts of this schema.
