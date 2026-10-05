# moneylens-converter-rs
**moneylens-converter-rs** is a Rust-based command-line tool that converts Excel and OpenDocument spreadsheets into a clean, normalized JSON format for use with the MoneyLens personal finance app.

## Usage

```bash
cargo run -- --input path/to/workbook.ods --output output.json --month 3
```

The optional `--month` flag accepts digits `1` through `12` and limits the exported JSON to that month's transactions. Categories and tags in the output are derived from the selected month's data only.

### Remapping

The spreadsheet doesn't always match the app's current data structure — e.g. a category that used to be a flat leaf (`Utilities`) may have since gained children in the app (`Electricity`, `Heating & Cooling`, `Water`), and the app only accepts leaf categories on a transaction. The optional `--remap <path>` flag points to a TOML file that rewrites raw spreadsheet values before the payload is built, organized by entity — `category` and `bank_account`, e.g.:

```bash
cargo run -- --input path/to/workbook.ods --output output.json --remap remap.toml
```

```toml
# remap.toml
[[remap.category]]
type = "spend"
from = "Utilities"
to = "Utilities/Other"

[[remap.bank_account]]
from = "X"
to = "Amex Platinum"
```

Each `[[remap.category]]` entry's `type` (`spend`, `save`, or `earn`), `from`, and `to` are all required. A transaction matches an entry when its type and (trimmed) category exactly equal that entry's `type`/`from`; matching is otherwise case-sensitive. `to` can itself be a `"Parent/Child"` path — the existing parent/child category splitting applies to the rewritten value, so no other configuration is needed to produce the right `categories[]` entries.

The file is rejected (with an error naming the problem) if any `[[remap.category]]` entry has a blank `from`/`to`, has `from` equal to `to`, has a `to` with more than one level of nesting, or if two entries share the same `(type, from)` pair. No flag, no file: behavior is unchanged.

### Bank account remapping

`[[remap.bank_account]]` entries have just a `from`/`to` — no `type`, since a bank account isn't scoped by transaction type the way categories are. They apply to the spreadsheet's single-letter bank account symbols (currently only read from the `Spend` sheet) *before* the existing hardcoded letter-to-name mapping, not after: a matching entry wins outright, and only an unmatched symbol falls back to the hardcoded mapping (`X` → `AmEx`, `B` → `Barclays`, `W` → `Wise Virtual`, `M` → `Monzo`, `A` → `Wise Physical`, any other letter → `Unknown`). A cell with no symbol at all is matched using the literal `from = "(empty)"` (the hardcoded default for a blank cell is `NatWest`) — e.g.:

```toml
[[remap.bank_account]]
from = "(empty)"
to = "Monzo"
```

Validated the same way as categories (blank `from`/`to`, `from == to`, duplicate `from`), minus the nesting-depth check, since bank account names have no parent/child structure. No flag, no file: behavior is unchanged.
