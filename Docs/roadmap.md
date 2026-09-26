# Roadmap — Implementation Phases

> **Status: Plan of record as of 2026-09-27. Sequenced, not scheduled.**
> Phases 1–8 contain no frontend work. See [frontend.md](frontend.md).
>
> **Platform: Linux for all development, building, and testing.** Windows is
> touched exactly once, at the end, to confirm the release binary runs — Phase 9.
> There is deliberately **no CI/build-delivery phase**: nothing is delivered to
> anyone until the project is finished, so a build pipeline would be plumbing for
> a single consumer who already has the source.

Each phase lists its **deliverables**, its **acceptance criterion** (the smallest
observable thing that proves it works), and what it **depends on**. A phase is
done when its acceptance criterion passes in the real environment, not when the
code compiles.

---

## Phase 1 — Foundations

**Deliverables**
- Dependencies: `sqlx` (sqlite, `runtime-tokio`, **no compile-time macros**),
  `csv`, `sha2`, `uuid` (v4), `serde_json`, `time`.
  Using the runtime query API means the build never needs a live DB or a
  checked-in `.sqlx/` directory — worth the small loss of compile-time checking.
- `src/db/mod.rs` — pool init, `PRAGMA journal_mode=WAL`,
  `PRAGMA foreign_keys=ON`, `sqlx::migrate!()`.
- `src/db/migrations/0001_init.sql` — the schema from
  [data-model.md](data-model.md), verbatim.
- Config additions: `APP_DB_PATH`, `IMPORTS_DIR`, `PICTURES_DIR`, `EXPORTS_DIR`,
  `NUMBER_LOCALE`.
- New `AppError` variants per [api-surface.md](api-surface.md), each wired into
  `status()` and `code()`.
- `GET /api/v1/health` extended to report DB reachability.

**Acceptance** — the app starts, creates `app.db` and the four directories, runs
migrations, and health reports the DB as reachable. Bruno can call it.

**Depends on** — nothing. This is now the first phase; there is no build or
delivery prerequisite on Linux.

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
screens, static assets via `ServeDir`, and the browser launch on startup.

**Acceptance** — the whole daily loop is completable in a browser with no
command line: import, page through items, bulk approve, export, download both
CSVs.

**Depends on** — Phases 1–7. This is the first phase that can slip without
blocking anything else, because the API is independently exercisable via Bruno.

---

## Phase 9 — Windows delivery

The only Windows work in the plan, and it is deliberately last.

**Deliverables**
- Cross-compile from Linux: `rustup target add x86_64-pc-windows-gnu` plus a
  mingw-w64 toolchain, then `cargo build --release --target x86_64-pc-windows-gnu`.
- Settle the `openssl` story **here, not earlier**: the default is the system
  OpenSSL the linker finds, which mingw does not reliably provide. If the link
  fails, add `ssh2 = { version = "0.9.6", features = ["vendored-openssl"] }` and
  rebuild. Doing it now would mean carrying a vendored OpenSSL build through
  every phase of development for no benefit.
- Copy `target/x86_64-pc-windows-gnu/release/rust-ntg.exe` to the Windows VM and
  run it. `GET /api/v1/health` answers.

**Acceptance** — the `.exe` starts on the VM and `GET /api/v1/health` answers,
with `.env` supplied by hand as on any other platform.

**Depends on** — Phase 8. Everything is finished before this is attempted; the
purpose is only to confirm the artifact runs, not to develop against it.

**Known risk, accepted** — `vendor` is recorded rather than treated as a blocker.
If cross-compilation proves painful, the fallback is building on the Windows VM
itself with the Rust toolchain installed. Either way it is a one-off, so it does
not justify front-loading.

---

## Not in scope

- **ERP GraphQL sync.** The endpoint is local and requires auth, so it is not
  currently actionable. Recorded in [api-surface.md](api-surface.md) so the shape
  is not lost.
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
                        9  windows delivery  ← once, at the end
```

## Related

- [overview.md](overview.md) — index of all documents.
- [open-decisions.md](open-decisions.md) — what could still change this plan.
