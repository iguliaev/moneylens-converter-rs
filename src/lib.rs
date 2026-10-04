use std::error::Error;

pub mod category_remap;
pub mod options;
pub mod parsers;
pub mod payload;

use category_remap::RemapEntry;
use options::Options;
use payload::types::Transaction;
use spreadsheet_ods::{self};

pub fn run(opts: Options) -> Result<(), Box<dyn Error>> {
    let workbook = spreadsheet_ods::read_ods(&opts.input).expect("Failed to read ODS file");
    log::info!("Workbook has {} sheets", workbook.num_sheets());

    let selected_month = opts.month;
    let remap_entries: Vec<RemapEntry> = match &opts.category_remap {
        Some(path) => category_remap::load(path)?,
        None => Vec::new(),
    };
    let mut payload_builder = payload::PayloadBuilder::default();

    for sheet in workbook.iter_sheets() {
        log::info!("Sheet: {}", sheet.name());

        if parsers::save::can_parse(sheet) {
            let mut transactions =
                filter_transactions_by_month(parsers::save::parse(sheet), selected_month);
            category_remap::apply(&mut transactions, &remap_entries);
            payload_builder = payload_builder.add_transactions(transactions);
        }

        if parsers::utils::sheet_matches_month_selection(sheet.name(), selected_month) {
            if parsers::earn::can_parse(sheet) {
                let mut transactions = parsers::earn::parse(sheet);
                category_remap::apply(&mut transactions, &remap_entries);
                payload_builder = payload_builder.add_transactions(transactions);
            }
            if parsers::spend::can_parse(sheet) {
                let mut transactions = parsers::spend::parse(sheet);
                category_remap::apply(&mut transactions, &remap_entries);
                payload_builder = payload_builder.add_transactions(transactions);
            }
        }
    }

    let payload = payload_builder.build();

    let json = serde_json::to_string_pretty(&payload).expect("Failed to serialize");

    match opts.output {
        Some(ref path) => std::fs::write(path, &json).expect("Failed to write output file"),
        None => println!("{json}"),
    };

    Ok(())
}

fn filter_transactions_by_month(
    transactions: Vec<Transaction>,
    selected_month: Option<u8>,
) -> Vec<Transaction> {
    transactions
        .into_iter()
        .filter(|tx| parsers::utils::date_matches_month(&tx.date, selected_month))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::types::{Payload, TransactionType};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_output_path(test_name: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("moneylens-{test_name}-{unique_suffix}.json"))
    }

    #[test]
    fn filters_transactions_before_building_payload_metadata() {
        let january_food = Transaction {
            date: "2025-01-15".to_string(),
            transaction_type: TransactionType::Spend,
            category: "Food".to_string(),
            bank_account: "NatWest".to_string(),
            amount: 10.0,
            tags: vec!["Groceries".to_string()],
            notes: None,
        };
        let february_salary = Transaction {
            date: "2025-02-01".to_string(),
            transaction_type: TransactionType::Earn,
            category: "Salary".to_string(),
            bank_account: "Checking".to_string(),
            amount: 1000.0,
            tags: vec!["Payroll".to_string()],
            notes: None,
        };

        let payload = payload::PayloadBuilder::default()
            .add_transactions(filter_transactions_by_month(
                vec![january_food, february_salary],
                Some(2),
            ))
            .build();

        assert_eq!(payload.transactions.len(), 1);
        assert_eq!(payload.transactions[0].date, "2025-02-01");
        assert_eq!(payload.categories.len(), 1);
        assert_eq!(payload.categories[0].name, "Salary");
        assert_eq!(payload.tags.len(), 1);
        assert_eq!(payload.tags[0].name, "Payroll");
        assert_eq!(payload.bank_accounts.len(), 1);
        assert_eq!(payload.bank_accounts[0].name, "Checking");
    }

    #[test]
    fn run_filters_savings_transactions_by_selected_month() {
        let output_path = temp_output_path("savings-month");

        run(Options {
            input: PathBuf::from("tests/data/savings_example.ods"),
            output: Some(output_path.clone()),
            month: Some(1),
            category_remap: None,
        })
        .expect("run should succeed");

        let json = fs::read_to_string(&output_path).expect("output json should be written");
        fs::remove_file(&output_path).expect("temporary output file should be removed");

        assert!(json.contains("\"tags\": []"));

        let payload: Payload = serde_json::from_str(&json).expect("payload should deserialize");

        assert_eq!(payload.transactions.len(), 2);
        assert!(
            payload
                .transactions
                .iter()
                .all(|transaction| transaction.date.starts_with("2025-01-"))
        );
        assert_eq!(payload.categories.len(), 1);
        assert_eq!(payload.categories[0].name, "Other");
        assert_eq!(payload.categories[0].parent, None);
        assert!(payload.tags.is_empty());
        assert!(
            payload
                .transactions
                .iter()
                .all(|transaction| transaction.tags.is_empty())
        );
    }

    #[test]
    fn run_applies_category_remap_end_to_end() {
        let output_path = temp_output_path("category-remap");
        let remap_path = std::env::temp_dir().join(format!(
            "moneylens-category-remap-e2e-{}.toml",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        fs::write(
            &remap_path,
            r#"
            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Other"
            "#,
        )
        .expect("writing temp remap file should succeed");

        run(Options {
            input: PathBuf::from("tests/data/spend_earn_transactions_example.ods"),
            output: Some(output_path.clone()),
            month: None,
            category_remap: Some(remap_path.clone()),
        })
        .expect("run should succeed");

        let json = fs::read_to_string(&output_path).expect("output json should be written");
        fs::remove_file(&output_path).expect("temporary output file should be removed");
        fs::remove_file(&remap_path).expect("temporary remap file should be removed");

        let payload: Payload = serde_json::from_str(&json).expect("payload should deserialize");

        assert!(
            payload
                .transactions
                .iter()
                .all(|transaction| transaction.category != "Utilities"),
            "no transaction should keep the pre-remap bare \"Utilities\" category"
        );
        assert!(
            payload
                .transactions
                .iter()
                .any(|transaction| transaction.category == "Utilities/Other"),
            "at least one transaction should have been remapped to \"Utilities/Other\""
        );

        let parent = payload
            .categories
            .iter()
            .find(|category| category.name == "Utilities")
            .expect("root \"Utilities\" category entry should exist");
        assert_eq!(parent.parent, None);

        let child = payload
            .categories
            .iter()
            .find(|category| category.name == "Other")
            .expect("\"Other\" child category entry should exist");
        assert_eq!(child.parent, Some("Utilities".to_string()));
    }
}
