# Field Mapping, Vocabulary and Hashing

> **Status: Decided 2026-09-26. In force.**
> The ERP header *vocabulary* is defined in code. The *instance* — which source
> column feeds which header, with which transforms, in which order — is data in
> the database, editable by the user.

## The Split: Vocabulary in Code, Instance in the Database

This is the central rule of the whole mapping design, and it resolves what
looked like a contradiction between two earlier statements:

- the ERP's headers are fixed and must be impossible to get wrong → **code**
- the user drops, combines and adds columns regularly → **database**

Both hold, at different layers.

```rust
// ── Code: the closed set of headers the ERP will ever see ───────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErpField {
    Ntg001, Ntg002, /* … */ Ntg998, Ntg999Meta,
}

impl ErpField {
    pub const ALL: &'static [Self] = &[ /* … */ ];

    /// The literal header text. This is the ONLY place a header string exists.
    pub const fn header(self) -> &'static str {
        match self {
            Self::Ntg001 => "NTG-001",
            /* … */
            Self::Ntg999Meta => "NTG-999",
        }
    }

    pub fn from_ntg(s: &str) -> Result<Self, AppError> { /* … */ }
}
```

```rust
// ── Database: which of those are active, fed by what, in what order ─────────
pub struct FieldConfig {
    pub id: u16,                 // slot number; output order is ORDER BY id
    pub ntg: ErpField,           // validated on write
    pub source: Source,
    pub ops: Vec<Op>,
    pub active: bool,
}
```

**Consequences:**

- The header *text* is never stored, so it cannot be wrong. A `field_config` row
  whose `ntg` does not parse to an `ErpField` is rejected at the API boundary.
- Adding a genuinely new kind of ERP column = new enum variant + recompile.
  Deliberate, reviewed, and rare.
- Dropping one, re-pointing it at a different source column, changing a
  transform, or reordering = a database edit, no recompile.
- There is deliberately **nowhere to put a free-text header**, which is exactly
  what would produce a broken ERP import.

## The `NTG-999` Problem, and Why Slots Are the Real Identity

Three metadata columns all carry the header `NTG-999`. They are only
distinguishable **by position**. Therefore `NTG-999` cannot be a primary key for
`field_config` — three rows would collide.

The resolution: **`field_config.id` (the slot number) is the identity.** `ntg` is
its label.

| | |
| --- | --- |
| Identity / primary key | `id` — the slot number, also the output order |
| Label | `ntg` → `ErpField::header()` |
| What `Source` and `Op` reference | the slot `id` |

The UI presents slots by their NTG label (`NTG-017 — title`), so the user never
sees a raw slot number. Internally, references are unambiguous even when three
slots share a label. This is flagged in [open-decisions.md](open-decisions.md) as
a deviation from "look everything up by NTG name" that follows necessarily from
the repeated `NTG-999` label.

The source-column index is stored separately in `source_columns.source_index`
and is what the *initial* positional drop uses.

## The Two-Stage Pipeline

```
raw CSV row (positional cells)
   │
   ├─ stage 1 — POSITIONAL: skip cells whose source_index is in the drop set
   │             (this is where source_index is used)
   │
   ├─ stage 2 — build a map: ErpField → value, from the surviving cells
   │
   └─ stage 3 — BY SLOT: for each active field_config, resolve Source and
               apply Ops in order
```

Stages 1 and 2 turn a positional file into a name-keyed row. Stage 3 is the
part the user configures, and it never depends on position.

This is why the source index is stored at all: the initial drop is naturally
expressed positionally ("skip source columns 12, 47, 88…"), and everything after
it is expressed by name. Storing the index is a precaution that makes the first
stage possible and makes a mis-mapped header diagnosable.

## `Source` — Where a Value Comes From

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// One slot, used as-is.
    Column { slot: u16 },
    /// Several slots joined — "combining more or less columns".
    Columns { slots: Vec<u16>, sep: String },
    /// A set of slots reduced to one value by an aggregate Op.
    Set { slots: Vec<u16> },
    /// A fixed value.
    Const { value: String },
}
```

## `Op` — Transformations

An ordered list applied per field. Each variant is a small, closed, validated
operation. **There is no expression parser and no `eval`** — arithmetic
requirements were confirmed to be string combination only.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Trim,
    Upper,
    Lower,
    Replace { from: String, to: String },
    DefaultIfEmpty { value: String },
    Truncate { len: usize },
    Pad { len: usize, fill: char },
    Concat,                              // valid only on Source::Columns
    Max,                                 // valid only on Source::Set
    Min,                                 // valid only on Source::Set
}
```

Validation rules, enforced in code when a `FieldConfig` is written:

- `Concat` only on `Source::Columns`
- `Max` / `Min` only on `Source::Set`
- every referenced `slot` must exist
- no cycles (a slot cannot reference itself)

**Ops are added on demand, not up front.** Each is 5–15 lines. Start with `Trim`,
`Upper`, `Lower`, `Replace`, `DefaultIfEmpty`, `Truncate`, `Concat`, `Max`,
`Min`; add more when a real need appears.

**Regex is deliberately excluded for now.** It is a compile cost per row across
100 000 rows, and a catastrophic-backtracking hazard on hostile input from an
external feed. Add `RegexReplace` if a real requirement appears, with a
complexity guard.

### Comparison Semantics

`Max` / `Min` compare **numerically with a lexicographic fallback**, because the
source data is very likely German-formatted (`1.234,56`). This is a hidden
surprise if left undocumented, so it is stated here and covered by a test.

```rust
/// Parses `1234.56`, `1.234,56` and `12,50` as numbers.
/// Returns `None` when the value is not numeric.
fn parse_number(s: &str) -> Option<f64>;
```

The ambiguity in `1.234` (English 1.234 vs German 1234) is resolved by
configuration, not by a clever heuristic — see `NUMBER_LOCALE` in
[deployment.md](deployment.md). Hardware and real-world formats need a
calibration knob, not a guess.

## Hashing: Exact Serialisation Matters

```rust
pub fn content_hash(projected: &[(ErpField, String)]) -> [u8; 16];
```

Hash **the `(ntg, value)` pairs in slot order**, each pair written as
`ntg_header` + `\x1f` + `value`, and the pairs concatenated. Then SHA-256,
truncated to 16 bytes.

Two mistakes this avoids, both of which produce bugs that are hard to see:

1. **Hashing a `serde_json::Map` or a `Value`.** Map iteration order is not
   guaranteed, so the same logical row can hash differently on different runs.
   Symptom: a few percent of items appear "changed" every import, forever.
2. **Hashing a bare `Vec<String>`.** Then `["ab", "c"]` and `["a", "bc"]`
   produce the same digest. The `\x1f` separator removes the ambiguity.

Reordering slots **does** change the hash. That is correct: slot order is the
output column order, the ERP reads by position, so a reorder is a real change
to the data being sent.

## Re-Evaluation Triggers

`profile.rev` is bumped whenever `field_config` changes. An item is stale when
`items.hashed_at_rev <> profile.rev`.

Stale items need re-evaluation, which means:

1. recomputing `content_hash` from the stored blob, and
2. refilling any newly added display columns.

Import does this for every row in the incoming file, automatically. It does
**not** do it for items absent from recent files — hence the explicit
`POST /api/v1/maintenance/re-evaluate` action. See
[open-decisions.md](open-decisions.md) for whether this is triggered explicitly
by the user or automatically on config save.

## The Positional-Import Risk (unresolved)

**The ERP imports by position.** Its import definition is an array of headers,
but matching is positional; identical headers exist so a human can debug a file.

The user's stated behaviour is **shift left** on drop: inactive slots are omitted
and everything after them moves up. That is only safe if the `NTG-999` metadata
slots are the **last** slots in the output order. If a metadata slot sits in the
middle and is dropped, every later field lands in the wrong slot — and because
the import is positional, **it succeeds with wrong data**. That is precisely the
error class the positional design makes invisible.

Two candidate resolutions, both recorded in
[open-decisions.md](open-decisions.md):

- **(a)** Hold positions: write an empty string at inactive slots. The file has
  blank columns; debugging stays honest. Never misaligns.
- **(b)** Keep shift-left, and assert in code that `Ntg999Meta` slots always sort
  last. Cleaner file, and the failure mode is removed by construction.

Until this is decided, `write.rs` must not assume either behaviour.

## Related

- [data-model.md](data-model.md) — where these live.
- [export.md](export.md) — where the projection is written out.
- [open-decisions.md](open-decisions.md) — the unresolved items in this document.
