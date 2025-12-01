use super::utils;
use crate::payload::types::{Transaction, TransactionType};
use spreadsheet_ods::Sheet;

pub fn parse(sheet: &Sheet) -> Vec<Transaction> {
    assert!(
        utils::is_month(sheet.name()),
        "Expected month name, got: {}",
        sheet.name()
    );

    const FIRST_DATA_ROW: u32 = 2;
    const MAX_ROWS: u32 = 1000;

    // Column indices for the Spend sheet
    const COL_DATE: u32 = 5;
    const COL_CATEGORY: u32 = 6;
    const COL_AMOUNT: u32 = 7;
    const COL_BANK_ACCOUNT: u32 = 8;
    const COL_TAGS: u32 = 9;

    let mut empty_date_count = 0;
    const EMPTY_DATE_THRESHOLD: usize = 5;

    let mut transactions = Vec::new();

    for row_idx in FIRST_DATA_ROW..(FIRST_DATA_ROW + MAX_ROWS) {
        // Check if we've reached the end of data
        let Some(date) = utils::extract_date(sheet, row_idx, COL_DATE) else {
            empty_date_count += 1;
            if empty_date_count >= EMPTY_DATE_THRESHOLD {
                break;
            }
            continue;
        };

        // Reset empty counter when we find a valid date
        empty_date_count = 0;

        // Extract and validate required fields
        let Some(amount) = utils::extract_amount(sheet, row_idx, COL_AMOUNT) else {
            eprintln!("Warning: Skipping row {row_idx} - missing amount");
            continue;
        };

        let Some(category) = utils::extract_text(sheet, row_idx, COL_CATEGORY) else {
            eprintln!("Warning: Skipping row {row_idx} - missing category");
            continue;
        };

        let bank_account_symbol = utils::extract_text(sheet, row_idx, COL_BANK_ACCOUNT);
        let bank_account = utils::bank_account_symbol_to_name(bank_account_symbol);

        let tag = utils::extract_text(sheet, row_idx, COL_TAGS);
        let annotation = utils::extract_annotation(sheet, row_idx, COL_AMOUNT);

        println!(
            "Date: {date}, Amount: {amount}, Category: {category}, Bank Account: {bank_account}, Tags: {tag:?}, Annotation: {annotation:?}",
        );

        let mut tags = Vec::new();
        if let Some(tag_value) = tag {
            tags.push(tag_value);
        }

        let transaction = Transaction {
            date,
            type_: TransactionType::Spend,
            category,
            bank_account,
            amount,
            notes: annotation,
            tags: if tags.is_empty() { None } else { Some(tags) },
        };

        transactions.push(transaction);
    }

    transactions
}
