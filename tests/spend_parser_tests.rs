use moneylens_converter_rs::{parsers, payload::types::TransactionType};
use spreadsheet_ods;

#[test]
fn test_parse_earnings() {
    let workbook = spreadsheet_ods::read_ods("tests/data/spend_earn_transactions_example.ods")
        .expect("Failed to read ODS file");
    let sheet = workbook.sheet(0);
    let transactions = parsers::spend::parse(sheet);

    assert_eq!(9, transactions.len(), "Expected 9 transactions parsed");

    assert_eq!(transactions[0].date, "2025-01-01");
    assert_eq!(transactions[0].amount, 100.00);
    assert_eq!(transactions[0].category, "Food".to_string());
    assert_eq!(transactions[0].notes, None);
    assert_eq!(transactions[0].type_, TransactionType::Spend);
    assert_eq!(transactions[0].bank_account, "NatWest".to_string());

    assert_eq!(transactions[4].date, "2025-01-05");
    assert_eq!(transactions[4].amount, 500.00);
    assert_eq!(transactions[4].category, "Vacation".to_string());
    assert_eq!(transactions[4].notes, Some("Tickets".to_string()));
    assert_eq!(transactions[4].type_, TransactionType::Spend);
    assert_eq!(transactions[4].bank_account, "AmEx".to_string());

    assert_eq!(transactions[6].tags, Some(vec!["Gas".to_string()]));

    assert_eq!(transactions[8].date, "2025-01-10");
    assert_eq!(transactions[8].amount, 150.00);
    assert_eq!(transactions[8].category, "Food".to_string());
    assert_eq!(transactions[8].notes, None);
    assert_eq!(transactions[8].type_, TransactionType::Spend);
    assert_eq!(transactions[8].bank_account, "AmEx".to_string());
}
