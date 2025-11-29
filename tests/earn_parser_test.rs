use moneylens_converter_rs::{parsers, payload::types::TransactionType};
use spreadsheet_ods;

#[test]
fn test_parse_savings_sheet() {
    let workbook = spreadsheet_ods::read_ods("tests/data/spend_earn_transactions_example.ods")
        .expect("Failed to read ODS file");
    let sheet = workbook.sheet(0);
    let transactions = parsers::earn::parse(sheet);

    assert_eq!(2, transactions.len(), "Expected 2 transactions parsed");
}
