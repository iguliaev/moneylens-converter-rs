

use spreadsheet_ods::{Sheet};
use crate::payload::types::{Transaction, TransactionType};


fn extract_date(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::DateTime(dt) => Some(dt.date().to_string()),
        spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
        _ => None,
    }
}

fn extract_amount(sheet: &Sheet, row: u32, col: u32) -> Option<f64> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::Number(f) => Some(*f),
        spreadsheet_ods::Value::Currency(c, _) => Some(*c),
        _ => None,
    }
}

fn extract_category(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
        _ => None,
    }
}

fn extract_notes(sheet: &Sheet, row: u32, col: u32) -> Option<String> {
    match sheet.value(row, col) {
        spreadsheet_ods::Value::Empty => None,
        spreadsheet_ods::Value::Text(s) => Some(s.to_string()),
        _ => None,
    }
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
    let mut empty_date_count = 0;

    let mut transactions = Vec::new();

    for row_idx in 2..1000 {
        let date = extract_date(sheet, row_idx, 2);
        if date.is_none() {
            empty_date_count += 1;
            if empty_date_count >= EMPTY_DATE_THRESHOLD {
                break;
            }
            continue;
       }
       let amount = extract_amount(sheet, row_idx, 3).unwrap_or(0.0);
       let category = extract_category(sheet, row_idx, 4).unwrap_or("".to_string());
       let notes = extract_notes(sheet, row_idx, 5).unwrap_or("".to_string());
       
       println!("Date: {}, Amount: {}, Category: {}, Notes: {}", date.clone().unwrap(), amount, category, notes);
       
       let transaction = Transaction {
           date: date.clone().unwrap(),
           type_: TransactionType::Save,
           category,
           bank_account: BANK_ACCOUNT_NAME.to_string(),
           amount,
           tags: None,
           notes: if notes.is_empty() { None } else { Some(notes) },
       };
       transactions.push(transaction);
    }
    transactions
}
