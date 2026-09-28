# Open Decisions

> **Status: Living list. Items marked BLOCKING must be answered before the phase
> that depends on them starts.**

Nothing here is a defect. These are the points where the design was deliberately
left open, with the reasoning that got us to the edge of each one.

---

## 1. Inactive slots: shift left, or hold position? — **BLOCKING, highest severity**

**The problem.** The ERP **imports by position**. Its import definition is an
array of headers, but matching is positional; identical headers exist only so a
human can debug a file. The stated behaviour is to **shift left** — inactive
slots are omitted and later fields move up.

That is only safe if the `NTG-999` metadata slots are the **last** slots in the
output order. If a metadata slot sits in the middle and is dropped, every later
field lands in the wrong slot, and because the import is positional **it succeeds
with wrong data**. This is exactly the error class the positional design makes
invisible, and it is the single most dangerous item in the project.

| | Option |
| --- | --- |
| **(a)** | Write an empty string at inactive slots. Positions never move. The file has blank columns; debugging stays honest. Cannot misalign. |
| **(b)** | Keep shift-left, and assert in code that `Ntg999Meta` slots always sort last. Cleaner file; failure mode removed by construction. |

**Recommendation:** (b), plus a startup assertion. If the metadata slots are
genuinely last already, (b) costs one assertion and (a) is unnecessary.

**Blocks:** the writer in `src/csv/write.rs` must not assume either behaviour.

---

## 2. Identity: EAN or SKU, and what if both are missing?

The ERP upserts on `ARTNR` **or** `EAN`; it is not yet decided which, though EAN
is the likely candidate. Two things follow.

**2a. Which one, and does it have to match the ERP's key?** The only real
constraint is that the key in this program must be the same key the ERP upserts
on, so the CSV and the future GraphQL path agree. Both columns are stored
regardless, so this is a migration, not a backfill.

**2b. A row with neither EAN nor SKU.** Currently an **error**. The alternative is
a synthetic key (`"row-<n>"` within the file), which is stable within one file
but *not* across re-imports of a changed file — a re-ordering upstream would
create a second synthetic item for the same product. Given "either one should
always be present", erroring is the safer default. Confirm.

**3. What the GraphQL path will need.** With CSV, the **ERP** decides
new-vs-update via upsert. With GraphQL, **this program** must decide. That means
the identity decision cannot be deferred past the point where the DB is designed
— which is why it is in this document rather than a distant one.

**Updated 2026-09-27:** the sentence "the ERP's GraphQL endpoint is local and
requires auth, so it is not currently actionable" is no longer true. The Linux
server reaches the ERP's GraphQL endpoints, so the path is now actionable and
this is no longer a theoretical constraint. See item 13 for the transport choice
and item 15 for the credential. **The data decision in 2a is now on the critical
path**, not merely cheap-while-cheap.

---

## 4. `Max` / `Min` comparison semantics

Proposed default: **numeric with a lexicographic fallback**, because the source
data is likely German-formatted (`1.234,56`). The ambiguity in `1.234` — English
1.234 or German 1234 — is resolved by a `NUMBER_LOCALE` config value rather than
a heuristic.

Confirm: is the data German-formatted, and is lexicographic fallback the right
tiebreak for non-numeric values?

---

## 5. Does the app modify source files on the SFTP server?

`SFTP_DEFAULT_OUT_DIRECTORY` and `SFTP_DEFAULT_ARCHIVE_DIRECTORY` exist in the
config, but whether consumed files are moved, renamed or deleted remotely is
undecided.

**The interaction to watch:** if files are removed remotely, the `imports` ledger
keyed on `remote_path` can no longer see them and the catch-up diff becomes
meaningless for anything already gone. The local copy in `IMPORTS_DIR` is what
makes re-derivation possible in that case, so removing remote files is *probably*
safe — but it should be a decision, not an accident.

**Recommendation:** leave remote files alone. The local copy plus the ledger is
sufficient, and it is one fewer thing that can go wrong on someone else's server.

---

## 6. Picture filename scheme

Item UUID as the file stem, or the original filename from the feed?

- **UUID** dedupes naturally and keeps arbitrary characters from an external feed
  out of the filesystem. Not human-browsable.
- **Original filename** is readable but collides and needs sanitising.

**Recommendation:** UUID-based path, original name kept as a suffix for
readability: `PICTURES_DIR/<uuid>/<original>`. Unresolved because the ERP may
itself impose a naming convention.

---

## 7. Decisions are currently one-way

`undecided → approved | rejected`, with no way back. Confirmed as acceptable for
now.

Flagged because the cost of reversibility is one enum variant and one branch,
while the cost of *not* having it is that a mis-click on "approve all 8 000" is
unrecoverable with no recourse but a database edit. Low priority; not worth
building before the export path works.

---

## 8. When is re-evaluation triggered?

`profile.rev` is bumped on every field-config change, which makes some items'
`content_hash` stale. The user is told how many went stale; whether the
re-evaluation then runs automatically or waits for an explicit action is
undecided.

- **Automatic on save** — never serves stale data, but a config change over
  100 000 items becomes a long request.
- **Explicit action** — the user sees the count and chooses. Slower, safer.

**Recommendation:** explicit action, with the count shown. The motivating case for
re-evaluation is rare (a legal requirement changing what must be tracked), and an
unattended 100 000-row job triggered by an innocent config tweak is a bad
trade.

---

## 9. Display columns: which set, and who owns it

The fields shown in the decision UI are real columns, added by migration. Two
sub-questions:

- **Which fields?** Not yet enumerated. The display set and the ERP output set
  are deliberately separate, but the display set has not been listed.
- **Migration or JSON extraction?** Migration was chosen for typed, indexed
  columns at the cost of a backfill. Revisit only if the display set turns out to
  change often — at which point an expression index over a `tracked` JSON column
  is the alternative.

---

## 10. Slot identity vs NTG name — a forced deviation

`field_config.id` (the slot number) is the identity, not the NTG code, because
**three metadata columns all carry the header `NTG-999`** and cannot be
distinguished by name. `Source` and `Op` therefore reference slot ids; the UI
displays them as NTG labels so the user still thinks in NTG names.

This follows necessarily from the repeated label, but it is a deviation from
"look everything up by NTG name" and should be confirmed rather than assumed.

---

## 11. ERP CSV dialect

Delimiter, encoding, quoting and line endings can be anything that satisfies CSV
rules. **Not yet confirmed:** the exact column set the ERP expects, and whether
any row grouping or multi-file-per-item behaviour is required. Only the headers
are fixed, and they come from `ErpField`.

---

## 12. SFTP session timeout behaviour

`Session::set_timeout` applies **per operation**. An import of 100 000 rows makes
many operations on one session, so a total-workload timeout can fire mid-import.

Decide during implementation: a `keepalive` interval instead of a generous
timeout, and whether a timeout means "reconnect and continue from the last
committed row" or "fail this file".

---

## 13. ERP delivery: CSV file or GraphQL push?

Added 2026-09-27. This is the largest remaining fork in the project, and the
deployment change made it urgent.

**What changed.** Until 2026-09-27 the ERP was assumed to be local to the Windows
VM, so an items CSV written next to the ERP was the obvious delivery mechanism
and a GraphQL push was blocked on "the endpoint is local and requires auth". The
Linux server has since been confirmed to **reach the ERP's GraphQL endpoints**, so
that blocker is gone. The original argument for the CSV — that a file avoids
transferring anything between the two machines — is also gone, because items never
need to cross between the machines either way: the Linux box pulls the source CSV
from the external SFTP server and pushes to the ERP.

| | CSV file | GraphQL push |
| --- | --- | --- |
| Who decides new-vs-update | **The ERP**, via upsert | **This program** |
| Auth | none needed locally | an ERP credential in config |
| Partial failure | re-import the file; idempotent | per-item retry, needs its own bookkeeping |
| `open-decisions.md` item 1 (positional vs shift-left) | **still applies** — the ERP reads by position | **disappears** — a mutation has named fields |
| `open-decisions.md` item 11 (CSV dialect) | **still applies** | **disappears** |
| Debuggability | open the file and look | inspect the request/response |

**Recommendation: GraphQL.** It eliminates the two most dangerous open items in
the project (items 1 and 11) outright, because there is no column order to get
wrong when the fields are named. It also removes the "the import succeeded with
wrong data" failure class, which is the single worst thing that can happen to this
system. The cost is that this program becomes responsible for new-vs-update
decision, which is item 2a's identity question — and that question is cheap to
answer now and expensive later regardless of transport.

**Not blocking Phase 6.** The selection predicate, `content_hash`, and the
`last_exported_hash` bookkeeping are identical for both transports, so the export
phase can be built once against a small transport seam.

**Also open under GraphQL:** how pictures reach the ERP, which is item 14.

---

## 14. How do pictures reach the ERP?

Added 2026-09-27. Deferred — explicitly **not urgent**, and recorded so it is a
decision rather than an omission.

If item 13 resolves to a GraphQL push, the picture files are a separate problem.
`exports.pictures_csv_path` implies the ERP is currently handed a CSV of picture
*paths*; but the pictures are binaries, and the Linux server's filesystem is not
the ERP's filesystem, so a path is not a delivery mechanism any more.

Options, roughly in order of preference:

| | Option |
| --- | --- |
| (a) | **Base64 or multipart inside the GraphQL mutation.** No extra infrastructure. Bloats requests and the ERP's memory at 100 000 items. |
| (b) | **`robocopy` from Windows to an SMB share on the Linux server**, or the reverse direction if the ERP reads from the share. This is the current expectation, and the reachability is confirmed. Needs a share, credentials, and a scheduled job. |
| (c) | **Serve pictures over HTTP and let the ERP fetch them by URL.** The app already needs a download endpoint for the UI. Cleanest if the ERP will do the fetching; unknown until its capabilities are checked. |

**Recommendation:** (c) if the ERP can fetch, otherwise (b). Avoid (a) at this
scale. The deciding question is what the ERP's GraphQL API actually offers for
binary payloads, which has not been checked.

---

## 15. ERP GraphQL credential

Added 2026-09-27. Not open in the sense of undecided — **confirmed**: the ERP
requires authentication, and the Linux server can reach it.

So this is a to-do with a shape rather than a question. The credential needs the
same treatment as the SFTP password:

- Its own config block, mirroring `SftpConfig` — a hand-written `Debug` that
  redacts the secret. A derived `Debug` on a credential is a leak into logs.
- Same rule as `DatabaseConfig`: `DATABASE_URL` now carries a password too, and
  needs the same redaction.
- `.env` only, never committed. `.gitignore` already covers it.

**Recommendation:** one shared redaction approach rather than a hand-written
`Debug` per block — a small newtype wrapping a `SecretString`-style value with a
`Debug` that prints `***` is less error-prone than remembering to do this three
times. Revisit when the third one is written.

---

## 16. Migration strategy

Added 2026-09-27. **Being decided now** — the first migration is being written
immediately after this document set, so this is expected to resolve quickly.

Undecided, and the shape of the answer depends on how many migrations there will
ever be. This is a prototype with a single consumer, so "how do we manage
migrations at scale" is the wrong question. The live questions are narrower:

- Does the display-column set get enumerated before `0001`, or added as
  `0002`? Adding a display column later is `ALTER TABLE ... ADD COLUMN` plus a
  backfill, and on PostgreSQL 11+ with a non-volatile default that is cheap.
- Is the `blob` column `TEXT` or `JSONB`? Stated principle says `TEXT`, because
  the blob is never queried and `JSONB`'s main advantage is indexing.
- `time` or `chrono` for the `TIMESTAMPTZ` mapping? `Cargo.toml` currently enables
  `chrono`; [data-model.md](data-model.md) and the roadmap plan `time`.

**Recommendation:** write `0001` with the six tables and nothing speculative.
There is no reason to pre-create columns for display fields that have not been
enumerated, and PostgreSQL makes adding them cheap later.

---

## Related

- [data-model.md](data-model.md) — the schema these decisions affect.
- [deployment.md](deployment.md) — the target that made items 13 and 14 live.
- [export.md](export.md) — the CSV transport that item 13 may replace.
- [field-mapping.md](field-mapping.md) — item 1 and item 10 live here.
- [roadmap.md](roadmap.md) — the phase plan these decisions sit in.
