use super::utils;
use crate::payload::types::{Transaction, TransactionType};
use spreadsheet_ods::Sheet;

pub fn can_parse(sheet: &Sheet) -> bool {
    sheet.name() == "Savings"
}

pub fn parse(sheet: &Sheet) -> Vec<Transaction> {
    assert!(sheet.name() == "Savings", "Expected sheet named 'Savings'");

    //     Date	|Amount	   |Category |	Notes
    // 2025-01-06	|150.00	   |Other	 |
    // 2025-01-26	|151.71	   |Other	| Interest
    // 2025-02-06	|150.00	   |Other	| Interest
    // 2025-02-09	|1,000.00. |Other	|

    const EMPTY_DATE_THRESHOLD: usize = 5;
    const BANK_ACCOUNT_NAME: &str = "Default Account";
    const FIRST_DATA_ROW: u32 = 2;
    const MAX_ROWS: u32 = 1000;

    // Column indices for the Savings sheet
    const COL_DATE: u32 = 2;
    const COL_AMOUNT: u32 = 3;
    const COL_CATEGORY: u32 = 4;
    const COL_NOTES: u32 = 5;

    let mut empty_date_count = 0;
    let mut transactions = Vec::new();

    for row_idx in FIRST_DATA_ROW..MAX_ROWS {
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

        // Extract optional fields
        let notes = utils::extract_text(sheet, row_idx, COL_NOTES);

        println!("Date: {date}, Amount: {amount}, Category: {category}, Notes: {notes:?}",);

        let transaction = Transaction {
            date,
            type_: TransactionType::Save,
            category,
            bank_account: BANK_ACCOUNT_NAME.to_string(),
            amount,
            tags: None,
            notes,
        };
        transactions.push(transaction);
    }
    transactions
}
