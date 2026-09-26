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
— which is why it is in this document rather than a distant one. The ERP's
GraphQL endpoint is local and requires auth, so it is not currently actionable;
the *data* decision is still cheap now and expensive later.

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

## Related

- [data-model.md](data-model.md) — the schema these decisions affect.
- [field-mapping.md](field-mapping.md) — item 1 and item 10 live here.
