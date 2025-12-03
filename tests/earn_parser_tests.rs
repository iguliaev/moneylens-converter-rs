use moneylens_converter_rs::{parsers, payload::types::TransactionType};
use spreadsheet_ods;

#[test]
fn test_parse_earnings() {
    let workbook = spreadsheet_ods::read_ods("tests/data/spend_earn_transactions_example.ods")
        .expect("Failed to read ODS file");
    let sheet = workbook.sheet(0);
    let transactions = parsers::earn::parse(sheet);

    assert_eq!(2, transactions.len(), "Expected 2 transactions parsed");
    assert_eq!(transactions[0].date, "2025-01-31");
    assert_eq!(transactions[0].amount, 4000.00);
    assert_eq!(transactions[0].category, "Salary".to_string());
    assert_eq!(transactions[0].notes, None);
    assert_eq!(transactions[0].type_, TransactionType::Earn);

    assert_eq!(transactions[1].date, "2025-01-31");
    assert_eq!(transactions[1].amount, 500.00);
    assert_eq!(transactions[1].category, "Bonus".to_string());
    assert_eq!(transactions[1].notes, Some("Earn Annotation".to_string()));
    assert_eq!(transactions[1].type_, TransactionType::Earn);
}
