# moneylens-converter-rs
**moneylens-converter-rs** is a Rust-based command-line tool that converts Excel and OpenDocument spreadsheets into a clean, normalized JSON format for use with the MoneyLens personal finance app.

## Usage

```bash
cargo run -- --input path/to/workbook.ods --output output.json --month 3
```

The optional `--month` flag accepts digits `1` through `12` and limits the exported JSON to that month's transactions. Categories and tags in the output are derived from the selected month's data only.

### Category remapping

The spreadsheet's category column doesn't always match the app's current category structure — e.g. a category that used to be a flat leaf (`Utilities`) may have since gained children in the app (`Electricity`, `Heating & Cooling`, `Water`), and the app only accepts leaf categories on a transaction. The optional `--category-remap <path>` flag points to a TOML file that rewrites bare category names before the payload is built:

```bash
cargo run -- --input path/to/workbook.ods --output output.json --category-remap remap.toml
```

```toml
# remap.toml
[[remap]]
type = "spend"
from = "Utilities"
to = "Utilities/Other"
```

Each `[[remap]]` entry's `type` (`spend`, `save`, or `earn`), `from`, and `to` are all required. A transaction matches an entry when its type and (trimmed) category exactly equal that entry's `type`/`from`; matching is otherwise case-sensitive. `to` can itself be a `"Parent/Child"` path — the existing parent/child category splitting applies to the rewritten value, so no other configuration is needed to produce the right `categories[]` entries.

The file is rejected (with an error naming the problem) if any entry has a blank `from`/`to`, has `from` equal to `to`, or if two entries share the same `(type, from)` pair. No flag, no file: behavior is unchanged.
