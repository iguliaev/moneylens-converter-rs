use moneylens_converter_rs::{parsers, payload::types::TransactionType};
use spreadsheet_ods;

#[test]
fn test_parse_savings_sheet() {
    let workbook = spreadsheet_ods::read_ods("tests/data/savings_example.ods")
        .expect("Failed to read ODS file");
    let sheet = workbook.sheet(0);
    let transactions = parsers::save::parse(sheet);

    assert_eq!(5, transactions.len());

    assert_eq!(transactions[0].date, "2025-01-01");
    assert_eq!(transactions[0].amount, 100.00);
    assert_eq!(transactions[0].category, "Other".to_string());
    assert_eq!(transactions[0].notes, Some("Note 01".to_string()));
    assert_eq!(transactions[0].type_, TransactionType::Save);

    assert_eq!(transactions[4].date, "2025-03-01");
    assert_eq!(transactions[4].amount, 500.00);
    assert_eq!(transactions[4].category, "Other".to_string());
    assert_eq!(transactions[4].notes, None);
    assert_eq!(transactions[4].type_, TransactionType::Save);
}
