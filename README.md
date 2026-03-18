# moneylens-converter-rs
**moneylens-converter-rs** is a Rust-based command-line tool that converts Excel and OpenDocument spreadsheets into a clean, normalized JSON format for use with the MoneyLens personal finance app.

## Usage

```bash
cargo run -- --input path/to/workbook.ods --output output.json --month 3
```

The optional `--month` flag accepts digits `1` through `12` and limits the exported JSON to that month's transactions. Categories and tags in the output are derived from the selected month's data only.
