# Roadmap — Implementation Phases

> **Status: Plan of record as of 2026-09-27. Sequenced, not scheduled.**
> Revised 2026-09-27 for the Linux deployment target and the PostgreSQL decision.
> Phases 1–8 contain no frontend work. See [frontend.md](frontend.md).
>
> **Platform: Linux, for development, building, testing and deployment.** The
> target is now a Linux server, so the development platform and the deployment
> platform are the same and Phase 9 is a service install rather than a
> cross-compile. Windows is used only as a **browser** — nothing is built or run
> there. There is deliberately **no CI/build-delivery phase**: nothing is
> delivered to anyone until the project is finished, so a build pipeline would be
> plumbing for a single consumer who already has the source.

Each phase lists its **deliverables**, its **acceptance criterion** (the smallest
observable thing that proves it works), and what it **depends on**. A phase is
done when its acceptance criterion passes in the real environment, not when the
code compiles.

---

## Phase 1 — Foundations

**Deliverables**
- Dependencies: `sqlx` (**postgres**, `runtime-tokio`, `macros`, `migrate`),
  `csv`, `sha2`, `uuid` (v4), `serde_json`, `time`.
  **Compile-time macros are in**, reversing the earlier runtime-only decision —
  see the note below this list.
- A local PostgreSQL for development, and `DATABASE_URL` in `.env`.
- `src/db/connection.rs` — pool init and `sqlx::migrate!()`. No `PRAGMA`: there is
  no journal mode to set and no `foreign_keys` pragma to remember, because
  PostgreSQL enforces foreign keys by default.
- `src/db/mod.rs` — `pub type DbPool = PgPool;`, used everywhere instead of naming
  the concrete pool. See "Keep the engine swappable" below.
- `src/db/migrations/0001_init.sql` — the schema from
  [data-model.md](data-model.md), in PostgreSQL dialect. **Being written now**;
  it will supersede the schema block in that document.
- A `justfile` with `sqlx` (regenerate `.sqlx/`) and `check` (verify, then clippy
  and test), where `build` depends on `check` — so the query cache is verified the
  same way tests are, before anything is built.
- `SQLX_OFFLINE=true` in `.env` and `.env.example`, and `.sqlx/` **committed**.
- Config additions: `IMPORTS_DIR`, `PICTURES_DIR`, `EXPORTS_DIR`, `NUMBER_LOCALE`.
  Note `APP_DB_PATH` is gone: `DATABASE_URL` replaces it.
- New `AppError` variants per [api-surface.md](api-surface.md), each wired into
  `status()` and `code()`. One `sqlx::Error` variant only — sqlx already provides
  `From<MigrateError> for sqlx::Error`, so a separate `Migration` variant is
  redundant.
- `GET /api/v1/health` extended to report DB reachability **with a real query**
  (e.g. `SELECT 1`), not `pool.is_closed()`, which is a purely local flag and
  would report "reachable" for a database that is gone.

**Acceptance** — the app starts, connects to PostgreSQL, runs migrations, and
health reports the DB as reachable by actually querying it. Bruno can call it.

**Depends on** — nothing. This is now the first phase; there is no build or
delivery prerequisite on Linux.

### Why compile-time macros, reversing the earlier decision

The 2026-09-27 draft said: no macros, runtime queries only, "worth the small loss
of compile-time checking" so the build never needs a live database. **Reversed.**
The reasoning was sound in isolation and wrong in context:

- The build now runs on the same platform it deploys to, so the "Windows
  cross-compile needs a live database" pressure that motivated it is gone. The
  previous version of this document even noted `cargo sqlx prepare` "will not be
  needed on any of these routes" — that was true, and it was also an argument for
  avoiding a real cost rather than a reason the cost did not exist.
- Every query is checked against a real schema at build time, and the return types
  are known to the compiler. For a project whose central risk is a data-modelling
  mistake, that is worth more than the convenience of `cargo build` working with
  no database.
- The cost is now one checked-in directory and one command. The gate is
  `cargo sqlx prepare --check`, run by `just check` before a build. What it
  catches and what it does not is recorded in
  [data-model.md](data-model.md) — including that it only *warns* on stale
  metadata from deleted queries, and passes vacuously before any query exists.

### Keep the engine swappable without designing for it

No trait, no factory, no engine enum. Two rules, and that is the whole of it:

1. **All SQL lives in `src/db/`.** Never inline a query string in a handler.
2. **Nothing outside `src/db/` names a concrete engine type.** `AppState` holds
   `DbPool`, not `PgPool`.

With those, changing engine is a change to `Cargo.toml` features,
`src/db/connection.rs`, and the migration DDL. That discipline is worth keeping
even though the engine is now decided, because it is what makes a decision like
this one reversible at all — and this one was reversed once already.

---

## Phase 2 — Domain model

**Deliverables**
- `src/domain/field.rs` — `ErpField` enum with `header()` and `from_ntg()`,
  `Source`, `Op`, `FieldConfig`, and validation (op-vs-source-variant, slot
  references exist, no self-reference).
- `src/domain/item.rs` — `Item`, `Decision`, `PictureState`.
- `src/domain/import.rs` — `ImportRecord`, `ImportStatus`.
- `src/domain/export.rs` — `ExportRecord`.
- `parse_number` with the `NUMBER_LOCALE` behaviour, and its tests.

**Acceptance** — `cargo test` passes for the serialisation round-trip
(`FieldConfig` ↔ JSON), the op/source validation rejects an invalid pairing, and
`parse_number` handles `1234.56`, `1.234,56`, `12,50` and non-numeric input.

**Depends on** — Phase 1.

**Note** — this is the vocabulary half of the code/data split. Adding an ERP
column is a new `ErpField` variant from here on, by design.

---

## Phase 3 — The pure core

The only part of the system that genuinely needs tests, because it is the only
part with real logic and no I/O.

**Deliverables**
- `src/csv/parse.rs` — streaming `csv::Reader` with a **reused `ByteRecord`**,
  configurable delimiter / quote / `has_headers`, `flexible(true)`. Reads the
  header row, reconciles it with `source_columns`, and turns each data row into
  a name-keyed map via the two-stage pipeline in
  [field-mapping.md](field-mapping.md).
- `src/csv/project.rs` —
  `project(row, field_config) -> Vec<(ErpField, String)>` and
  `content_hash(projected) -> [u8; 16]`, hashed over `\x1f`-separated
  `(ntg, value)` pairs in slot order.
- `src/csv/write.rs` — streaming items writer and picture writer.
- A small fixture CSV plus expected projections.

**Acceptance** — tests cover: a 400-column row parses without materialising the
file; a row differing only in a dropped column produces an **identical** hash;
`["ab","c"]` and `["a","bc"]` produce different hashes; a ragged row does not
abort the parse.

**Depends on** — Phase 2.

---

## Phase 4 — Import

**Deliverables**
- `POST /api/v1/imports`, `GET /api/v1/imports`, `GET /api/v1/imports/{id}`.
- `src/db/imports.rs`, `src/db/items.rs` (upsert + natural-key matching),
  `src/db/columns.rs` (discovery, deprecation, rename).
- One SFTP session for the whole work list — **not** the pool in
  `src/sftp/pool.rs`. That pool is request-concurrency machinery and is the wrong
  lifetime for a batch job; see [import.md](import.md).
- `GET /api/v1/source-columns` + deprecate/restore.

**Acceptance** — importing the same file twice changes nothing the second time
and records it as `unchanged`. Importing a file where one row's EAN is unchanged
but whose SKU changed **updates** that row rather than inserting a second one. A
file containing a duplicate key fails atomically: its ledger row is `failed`, and
no partial rows survive. Deleting a ledger row and re-importing reproduces the
same item ids.

**Depends on** — Phase 3.

**Open** — [open-decisions.md](open-decisions.md) item 12 (session timeout) is
resolved during this phase, not before it.

---

## Phase 5 — Decisions and item list

**Deliverables**
- `GET /api/v1/items` with **keyset** pagination, filters for `status` and
  `import_id`.
- `GET /api/v1/items/{id}` including the full blob for the detail view.
- `POST /api/v1/items/decisions` — filter in, count out, set-based `UPDATE`.

**Acceptance** — a bulk approve of 10 000 items is one request and one SQL
statement; paging through the list while decisions change underneath does not
skip or repeat a row.

**Depends on** — Phase 4.

---

## Phase 6 — Export

**Deliverables**
- `POST /api/v1/exports`, `GET /api/v1/exports`, the two CSV download endpoints,
  `POST /api/v1/exports/{id}/retry-pictures`.
- `src/db/exports.rs`.
- Picture download for approved items, with existence checks and per-item
  `picture_state`.

**Acceptance** — exporting twice in a row sends an empty items CSV the second
time. Changing a field config and re-exporting sends **only** the items whose
projection actually changed. A picture download failure still produces a full
items CSV and appears in `failed_pictures`. A generated CSV at 100 000 rows does
not spike memory.

**Depends on** — Phase 5.

**BLOCKED BY** — [open-decisions.md](open-decisions.md) item 1. The writer cannot
be written correctly until "shift left" vs "hold position" is answered.

---

## Phase 7 — Field config and maintenance

**Deliverables**
- `GET/PUT /api/v1/fields`, `GET /api/v1/fields/vocabulary`.
- `POST /api/v1/maintenance/re-evaluate` — rehash and backfill in one pass.
- First display-column migration, as the worked example of the migration path.

**Acceptance** — a `PUT` with an unknown `ntg` is rejected with a field-level
error; a `PUT` with `Max` on a `Source::Column` is rejected; after a valid `PUT`
the response reports the stale item count, and re-evaluation clears it.

**Depends on** — Phase 6.

---

## Phase 8 — Frontend

**Deliverables** per [frontend.md](frontend.md): `askama` templates, the six
screens, static assets via `ServeDir`. The app **logs its URL** rather than
opening a browser — the operator's browser is on a different machine, so opening
one is not something the app can do.

**Acceptance** — the whole daily loop is completable in a browser with no
command line: import, page through items, bulk approve, export, download both
CSVs.

**Depends on** — Phases 1–7. This is the first phase that can slip without
blocking anything else, because the API is independently exercisable via Bruno.

---

## Phase 9 — Linux delivery

The only deployment work in the plan, and it is deliberately last.

**Deliverables**
- A release binary built for Linux and a `systemd` unit that runs it.
- **Settle the `openssl` story here, not earlier.** The default is the system
  OpenSSL the linker finds. On a Linux target this is nearly always satisfied by
  the distribution's `libssl-dev`; if the link fails, add
  `ssh2 = { version = "0.9.6", features = ["vendored-openssl"] }` and rebuild.
  Doing it now would mean carrying a vendored OpenSSL build through every phase
  of development for no benefit.
- PostgreSQL installed, with a role and database created, and `pg_dump` in the
  backup story.
- nginx configured to terminate TLS and require authentication before forwarding
  to `127.0.0.1:8080`. Per [api-bind-address.md](api-bind-address.md) this is
  part of the security boundary and is reviewed as such.
- A `systemd` timer for the nightly import, per [deployment.md](deployment.md).

**Acceptance** — the service starts on the Linux server, `GET /api/v1/health`
answers through `https://<server>` from the Windows browser, and health reports
the database as reachable.

**Depends on** — Phase 8. Everything is finished before this is attempted.

**What dropped out of this phase.** The previous version of this phase was a
Windows cross-compile: `x86_64-pc-windows-gnu`, mingw-w64, Perl, and a
deliberate decision about vendoring OpenSSL for an *MSVC* target. All of that is
obsolete, because no Windows binary is produced. What remains is `cargo build
--release` on the target platform, a service file, and a `scp`. See
[deployment.md](deployment.md) for the full list of what was dropped and why.

**Known risk, accepted** — `vendored-openssl` is recorded rather than treated as
a blocker. If the system OpenSSL proves troublesome the fallback is enabling the
vendored feature. Either way it is a one-off, so it does not justify
front-loading.

---

## Not in scope

- **Picture transport to the ERP.** Pictures are binaries and do not travel over
  GraphQL; the current expectation is a `robocopy` from the Windows machine to an
  SMB share on the Linux server. Undecided and explicitly not urgent — see
  [open-decisions.md](open-decisions.md).
- **Moving, renaming or deleting files on the SFTP server** — undecided, see
  [open-decisions.md](open-decisions.md) item 5.
- **Row-level filtering** (dropping rows, not columns) — out of scope by
  decision. The pipeline drops columns only.

---

## Phase dependencies at a glance

```
1  foundations (db, config, errors)      ← current
   2  domain model (ErpField, Source, Op)
      3  pure core (parse, project, hash)  ← the only phase that needs real tests
         4  import
            5  decisions + list
               6  export        ← blocked by open-decision 1
                  7  field config + maintenance
                     8  frontend      ← can slip
                        9  linux delivery  ← once, at the end
```

Phase 6 currently specifies CSV delivery to the ERP. Whether that is replaced by
a GraphQL push is open — see [open-decisions.md](open-decisions.md) item 13. The
selection logic, hashing and the `last_exported_hash` bookkeeping are identical
either way, so the decision does not block Phase 6 from starting.

## Related

- [overview.md](overview.md) — index of all documents.
- [open-decisions.md](open-decisions.md) — what could still change this plan.
