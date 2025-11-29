
use super::utils;
use crate::payload::types::{Transaction, TransactionType};
use spreadsheet_ods::Sheet;

pub fn parse(sheet: &Sheet) -> Vec<Transaction> {
    assert!(utils::is_month(sheet.name()), "Expected month name, got: {}", sheet.name());

    const FIRST_DATA_ROW: u32 = 15;
    const MAX_ROWS: u32 = 100;

    // Column indices for the Savings sheet
    const COL_CATEGORY: u32 = 0;
    const COL_AMOUNT: u32 = 1;

    let mut transactions = Vec::new();

    transactions
}