# Plan: Align converter output with `bulk_upload_data` category schema

**Context**: `moneylens/docs/api/bulk-upload.md` was updated to add a `parent` field
on `CategoryInput`, used to express category hierarchy. This plan brings
`moneylens-converter-rs`'s JSON output in line with that schema.

Target repo: `moneylens-converter-rs` (this repo). All paths below are relative
to its root.

## Problem

The converter already emits transactions with `"category": "Vacation/Accomodation"`
— correct, since that's the `"Parent/Child"` path form the API expects for a
transaction's own `category` field. But it also emits this into the top-level
`categories` array:

```json
{ "name": "Vacation/Accomodation", "type": "spend", "description": null }
```

The updated API docs are explicit that this is now wrong:

> `name`: ... Always a plain leaf name — never a `"Parent/Child"` path (that
> convention is separate, used only by a transaction's own `category` field)

The new `parent` field is how hierarchy should be expressed in the `categories`
array instead:

```json
{ "type": "spend", "name": "Accomodation", "parent": "Vacation" }
```

Confirmed by inspecting local output artifacts (`test.json`,
`uk_expenses_2025.json`, etc. — these are untracked/gitignored-by-omission
local files, not part of the repo, generated from `.real_data/UK_EXPENSES_2025.ods`)
— they contain `categories[].name` values like `"Vacation/Accomodation"`,
`"Vacation/Groceries"`, etc.

There's also a second, independent bug found while investigating: category
dedup in `PayloadBuilder` is keyed only by name, not `(type, name)`. Since the
DB's uniqueness/lookup is `(user_id, type, name, parent_id)`, if the same bare
name is used under two different transaction types (e.g. `"Other"` appears for
both `save` and `earn` categories in real data), only the first gets a
`categories` entry today — transactions of the other type then fail
server-side with: `Category "Other" not found as a root-level category for
type "<type>"`.

## Step-by-step implementation

### 1. `src/payload/types.rs` — add `parent` field and make `TransactionType` hashable

`TransactionType` needs `Hash` because it will become part of a `HashSet` key
in the builder (step 2).

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Spend,
    Save,
    Earn,
}
```

Add `parent` to `Category`, optional and omitted from JSON when `None` (so
plain root categories serialize exactly as before, with no `"parent": null`
noise):

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Category {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}
```

`#[serde(default, ...)]` is required so existing/old JSON without a `parent`
key still deserializes (used by `lib.rs` integration tests that round-trip
JSON through `serde_json::from_str::<Payload>`). `PartialEq`/`Eq`/`Hash` are
required so `Category` itself can be used as the dedup key in step 2 — see the
note there for why.

### 2. `src/payload/builder.rs` — split `"Parent/Child"` category strings, and key dedup by `(type, parent, name)`

> **Note (post-review):** this section originally proposed a separate
> `HashSet<(TransactionType, Option<String>, String)>` tuple as the dedup key,
> shadowing `Category`'s own fields. After implementation, `Category` was made
> `PartialEq + Eq + Hash` instead (step 1) so the *same* `Category` value that
> gets pushed into `payload.categories` is also the `HashSet` element — no
> parallel key type. This relies on the builder always constructing categories
> with `description: None`; if that ever changes, dedup would need revisiting
> since `description` would then be part of the `Hash`/`Eq` identity too. The
> snippets below reflect what shipped.

```rust
#[derive(Default)]
pub struct PayloadBuilder {
    payload: Payload,
    category_set: std::collections::HashSet<super::types::Category>,
    bank_account_set: std::collections::HashSet<String>,
    tag_set: std::collections::HashSet<String>,
}
```

(`bank_account_set` and `tag_set` are unaffected — bank accounts and tags have
no `type` dimension and no hierarchy in the schema, so leave them as-is.)

Add a private helper that mirrors the API's own splitting rule ("split on the
*first* `/`, each side trimmed" — see `bulk-upload.md`'s `TransactionInput.category`
docs). It also has to defensively handle segments that are empty after
trimming (e.g. a stray leading/trailing `/`, or a cell that's pure whitespace)
and paths with more than one `/`, both found during code review — see the enum
variants' doc comments in `src/payload/builder.rs` for the exact rules:

```rust
enum CategorySplit {
    Root(String),
    Nested { parent: String, name: String },
    Multipath,
    Empty,
}

fn split_category(category: &str) -> CategorySplit { /* see src/payload/builder.rs */ }
```

Inside `add_transactions`, `match split_category(&tx.category)` builds the
`Category` entries for the `Root`/`Nested` cases (deduping via
`category_set.insert(category.clone())`), and for those same two cases
**normalizes `tx.category`** to the trimmed/reconstructed form (`name` for
`Root`, `"{parent}/{name}"` for `Nested`) so it's guaranteed to match the
`categories[]` entries emitted alongside it — the original plan said to leave
`tx.category` untouched, but code review pointed out that a spreadsheet cell
with incidental whitespace could otherwise produce a `categories[]` entry that
doesn't textually match the transaction's own `category` string. `Multipath`
and `Empty` log a warning (via `eprintln!`) and leave `tx.category` untouched,
since no matching `categories[]` entry is created for either — the API will
reject that transaction at upload time regardless, and preserving the raw
string aids debugging why.

### 3. Add/update tests in `src/payload/builder.rs`

Keep the existing `test_payload_builder` test as-is (it only uses flat
category names, unaffected by this change — just note it will now also
implicitly exercise `category_set`'s new `Category`-keyed dedup).

Add two new tests in the same `#[cfg(test)] mod tests` block (the shipped code
adds two further tests beyond these, covering the `Empty`/leading-slash edge
cases found in code review — see step 3.1 below):

```rust
#[test]
fn splits_slash_separated_category_into_parent_and_child_entries() {
    let transactions = vec![Transaction {
        date: "2025-01-10".to_string(),
        type_: TransactionType::Spend,
        category: "Vacation/Accomodation".to_string(),
        bank_account: "AmEx".to_string(),
        amount: 100.0,
        tags: vec![],
        notes: None,
    }];

    let payload = PayloadBuilder::default()
        .add_transactions(transactions.clone())
        .add_transactions(transactions) // re-adding must not duplicate entries
        .build();

    assert_eq!(payload.transactions.len(), 2);
    assert_eq!(
        payload.categories.len(),
        2,
        "expected exactly one parent entry and one child entry, deduped across both adds"
    );

    let parent = payload
        .categories
        .iter()
        .find(|c| c.name == "Vacation")
        .expect("parent category entry should exist");
    assert_eq!(parent.parent, None);
    assert_eq!(parent.type_, TransactionType::Spend);

    let child = payload
        .categories
        .iter()
        .find(|c| c.name == "Accomodation")
        .expect("child category entry should exist");
    assert_eq!(child.parent, Some("Vacation".to_string()));
    assert_eq!(child.type_, TransactionType::Spend);

    // the transaction's own `category` field must remain the full path, unchanged
    assert!(
        payload
            .transactions
            .iter()
            .all(|tx| tx.category == "Vacation/Accomodation")
    );
}

#[test]
fn same_bare_category_name_under_different_types_gets_separate_entries() {
    let transactions = vec![
        Transaction {
            date: "2025-01-01".to_string(),
            type_: TransactionType::Save,
            category: "Other".to_string(),
            bank_account: "Default Account".to_string(),
            amount: 10.0,
            tags: vec![],
            notes: None,
        },
        Transaction {
            date: "2025-01-02".to_string(),
            type_: TransactionType::Earn,
            category: "Other".to_string(),
            bank_account: "Default Account".to_string(),
            amount: 20.0,
            tags: vec![],
            notes: None,
        },
    ];

    let payload = PayloadBuilder::default()
        .add_transactions(transactions)
        .build();

    assert_eq!(
        payload.categories.len(),
        2,
        "same bare name under two different transaction types must produce two category entries"
    );
    assert!(
        payload
            .categories
            .iter()
            .any(|c| c.name == "Other" && c.type_ == TransactionType::Save)
    );
    assert!(
        payload
            .categories
            .iter()
            .any(|c| c.name == "Other" && c.type_ == TransactionType::Earn)
    );
}
```

### 4. `src/lib.rs` — no code change required, but strengthen one existing assertion

`run_filters_savings_transactions_by_selected_month` already asserts
`payload.categories[0].name == "Other"`. Optionally add
`assert_eq!(payload.categories[0].parent, None);` right after it, to lock in
that plain root categories still serialize with no `parent`. Not required —
the existing round-trip through `serde_json::from_str::<Payload>` will already
fail to compile/deserialize if the new field is wired up wrong, since
`Payload` derives `Deserialize`.

### 5. Parsers (`src/parsers/*.rs`) — no change needed

Parsers just extract raw cell text into `Transaction.category` verbatim
(whatever string is literally in the spreadsheet cell, e.g. `"Vacation"` or
`"Vacation/Accomodation"`). All splitting logic lives in the builder (step 2),
so `tests/spend_parser_tests.rs`, `earn_parser_tests.rs`, `save_parser_tests.rs`
need no changes.

### 6. Verification

Run, from the repo root:

```bash
cargo test
cargo build
```

All existing tests should still pass; the two new builder tests should pass.

### 7. Local output artifacts (optional, not a repo change)

`test.json`, `uk_expenses_2025.json`, `uk_expenses_04_2025.json`,
`uk_expenses_02_2025.json`, and `.real_data/` are **untracked** local files
(confirmed via `git status`), not part of the repo. There's nothing to
regenerate as part of this change. If you want to sanity-check the new output
shape against real data locally, run:

```bash
cargo run -- --input .real_data/UK_EXPENSES_2025.ods --output test.json
```

and confirm `categories[].name` no longer contains `/` and that
`Vacation`-style categories now show up as parent/child pairs with a `parent`
field on the child.

## Out of scope / known limitations

- `BankAccountInput`/`TagInput` shapes are already compatible — no changes.
- No 3-level category nesting exists in real data today, but code review
  (PR #8) flagged that `split_category` shouldn't silently mishandle it if it
  *does* occur. **Now handled**: a category string with more than one `/`
  (e.g. `"A/B/C"`) is detected as `CategorySplit::Multipath` — the schema caps
  hierarchy at 2 levels and can't represent it, so the converter logs a
  warning via `eprintln!` and skips adding any `categories[]` entries for that
  transaction, leaving `tx.category` untouched. The API will still reject that
  transaction at upload time (referenced category won't exist), but the
  converter itself no longer emits an invalid leaf name containing `/`.
- Similarly, code review found that a stray leading/trailing `/` (or a
  whitespace-only cell) could produce a `Category` with an **empty** `name`.
  **Now handled**: `split_category` collapses a segment that trims to nothing
  down to a plain root category using the other, non-empty segment (e.g.
  `"Bills/"` → `"Bills"`, `"  /Bills"` → `"Bills"`); if *both* segments are
  empty (e.g. `"/"`, `"   "`), that's `CategorySplit::Empty` — same
  warn-and-skip treatment as `Multipath`.
- Server-side "explicit entry wins over auto-derived parent" behavior (for
  `description`) doesn't apply here — the converter never sets category
  descriptions.

## Acceptance criteria

- [x] `Category` has a `parent: Option<String>` field, omitted from JSON when `None`.
- [x] `TransactionType` derives `Hash`; `Category` derives `PartialEq`/`Eq`/`Hash` too
      (shipped design — see the note in step 2).
- [x] `PayloadBuilder` splits any `"Parent/Child"` transaction category into a
      root parent `Category` entry + a child `Category` entry with `parent` set.
- [x] Category dedup identity is `(type, parent, name)` — implemented via
      `HashSet<Category>` rather than a separate tuple key (see step 2 note).
- [x] Transaction `category` fields are normalized (trimmed/reconstructed) to
      match their corresponding `categories[]` entries for the `Root`/`Nested`
      cases; left untouched for `Multipath`/`Empty` (deviates from the
      original "always leave unchanged" plan — see step 2 note).
- [x] Category strings with empty segments (stray `/`) or more than one `/`
      are detected and handled without emitting invalid `categories[]` entries.
- [x] New tests for all behaviors above pass; full test suite passes.
