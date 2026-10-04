use crate::payload::types::{Transaction, TransactionType};
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Deserialize, Debug, PartialEq)]
pub struct RemapEntry {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub from: String,
    pub to: String,
}

#[derive(Deserialize, Debug, Default)]
struct RemapFile {
    #[serde(default)]
    remap: Vec<RemapEntry>,
}

pub fn load(path: &Path) -> Result<Vec<RemapEntry>, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;
    let file: RemapFile = toml::from_str(&contents)?;
    Ok(file.remap)
}

/// Rewrites each transaction's `category` to the configured replacement when
/// its (type, category) exactly matches a remap entry's (type, from).
pub fn apply(transactions: &mut [Transaction], entries: &[RemapEntry]) {
    for transaction in transactions {
        if let Some(entry) = entries.iter().find(|entry| {
            entry.transaction_type == transaction.transaction_type
                && entry.from == transaction.category
        }) {
            transaction.category = entry.to.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction(transaction_type: TransactionType, category: &str) -> Transaction {
        Transaction {
            date: "2026-01-01".to_string(),
            transaction_type,
            category: category.to_string(),
            bank_account: "AmEx".to_string(),
            amount: 10.0,
            tags: vec![],
            notes: None,
        }
    }

    #[test]
    fn loads_remap_entries_from_toml() {
        let dir = std::env::temp_dir();
        let path = dir.join("moneylens-category-remap-test.toml");
        std::fs::write(
            &path,
            r#"
            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Other"
            "#,
        )
        .expect("writing temp remap file should succeed");

        let entries = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(
            entries,
            vec![RemapEntry {
                transaction_type: TransactionType::Spend,
                from: "Utilities".to_string(),
                to: "Utilities/Other".to_string(),
            }]
        );
    }

    #[test]
    fn rewrites_category_on_exact_type_and_name_match() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let mut transactions = vec![transaction(TransactionType::Spend, "Utilities")];
        apply(&mut transactions, &entries);

        assert_eq!(transactions[0].category, "Utilities/Other");
    }

    #[test]
    fn leaves_non_matching_categories_untouched() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let mut transactions = vec![transaction(TransactionType::Spend, "Groceries")];
        apply(&mut transactions, &entries);

        assert_eq!(transactions[0].category, "Groceries");
    }

    #[test]
    fn does_not_remap_across_different_transaction_types() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let mut transactions = vec![transaction(TransactionType::Earn, "Utilities")];
        apply(&mut transactions, &entries);

        assert_eq!(transactions[0].category, "Utilities");
    }
}
