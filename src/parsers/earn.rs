use super::utils;
use crate::payload::types::{Transaction, TransactionType};
use spreadsheet_ods::Sheet;

pub fn can_parse(sheet: &Sheet) -> bool {
    utils::is_month(sheet.name())
}

pub fn parse(sheet: &Sheet) -> Vec<Transaction> {
    assert!(
        utils::is_month(sheet.name()),
        "Expected month name, got: {}",
        sheet.name()
    );

    const FIRST_DATA_ROW: u32 = 14;
    const MAX_ROWS: u32 = 100;

    // Column indices for the Savings sheet
    const COL_CATEGORY: u32 = 0;
    const COL_AMOUNT: u32 = 1;

    const DATE_ROW: u32 = 1;
    const DATE_COL: u32 = 3;

    const DEFAULT_BANK_ACCOUNT_NAME: &str = "Default Account";

    let mut transactions = Vec::new();

    let date = utils::extract_date(sheet, DATE_ROW, DATE_COL)
        .unwrap_or_else(|| panic!("Failed to extract date from row {DATE_ROW}, col {DATE_COL}"));

    let mut empty_category_count = 0;
    const EMPTY_CATEGORY_THRESHOLD: usize = 5;

    for row_idx in FIRST_DATA_ROW..MAX_ROWS {
        // Extract and validate required fields
        let Some(category) = utils::extract_text(sheet, row_idx, COL_CATEGORY) else {
            empty_category_count += 1;
            if empty_category_count >= EMPTY_CATEGORY_THRESHOLD {
                break;
            }

            eprintln!("Warning: Skipping row {row_idx} - missing category");

            continue;
        };

        let Some(amount) = utils::extract_amount(sheet, row_idx, COL_AMOUNT) else {
            continue;
        };

        let annotation = utils::extract_annotation(sheet, row_idx, COL_AMOUNT);

        println!(
            "Date: {date}, Category: {category}, Amount: {amount}, Annotation: {annotation:?}"
        );

        let transaction = Transaction {
            date: date.clone(),
            type_: TransactionType::Earn,
            category,
            bank_account: DEFAULT_BANK_ACCOUNT_NAME.to_string(),
            amount,
            notes: annotation,
            tags: None,
        };

        transactions.push(transaction);
    }

    transactions
}
