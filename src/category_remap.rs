use crate::payload::types::{Transaction, TransactionType};
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RemapEntry {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub from: String,
    pub to: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct RemapFile {
    #[serde(default)]
    remap: Vec<RemapEntry>,
}

#[derive(Debug)]
struct InvalidRemapEntry {
    description: String,
}

impl fmt::Display for InvalidRemapEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid category remap entry: {}", self.description)
    }
}

impl Error for InvalidRemapEntry {}

/// Loads a `--category-remap` TOML file: a `[[remap]]` array of tables, each
/// with a `type` (`spend`/`save`/`earn`), `from`, and `to` bare category
/// name, e.g.:
///
/// ```toml
/// [[remap]]
/// type = "spend"
/// from = "Utilities"
/// to = "Utilities/Other"
/// ```
///
/// `from`/`to` are trimmed of surrounding whitespace. Rejects (as an error
/// naming the offending entry) a blank `from`/`to`, `from == to`, and two
/// entries sharing the same `(type, from)` — all of these would otherwise
/// be silently useless or ambiguous. An unrecognized key anywhere in the
/// file (e.g. a typo'd `[[remaps]]` table) is also rejected rather than
/// quietly parsing as zero entries.
pub fn load(path: &Path) -> Result<Vec<RemapEntry>, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read category remap file {}: {e}", path.display()))?;

    let file: RemapFile = toml::from_str(&contents).map_err(|e| {
        format!(
            "failed to parse category remap file {}: {e}",
            path.display()
        )
    })?;

    let entries: Vec<RemapEntry> = file
        .remap
        .into_iter()
        .map(|entry| RemapEntry {
            transaction_type: entry.transaction_type,
            from: entry.from.trim().to_string(),
            to: entry.to.trim().to_string(),
        })
        .collect();

    let mut seen = HashSet::new();
    for entry in &entries {
        if entry.from.is_empty() {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "\"from\" is blank for type \"{:?}\"",
                    entry.transaction_type
                ),
            }));
        }
        if entry.to.is_empty() {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "\"to\" is blank for type \"{:?}\", from \"{}\"",
                    entry.transaction_type, entry.from
                ),
            }));
        }
        if entry.from == entry.to {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "\"from\" and \"to\" are both \"{}\" for type \"{:?}\" — this entry is a no-op",
                    entry.from, entry.transaction_type
                ),
            }));
        }
        if !seen.insert((&entry.transaction_type, &entry.from)) {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "duplicate entry for type \"{:?}\", from \"{}\"",
                    entry.transaction_type, entry.from
                ),
            }));
        }
    }

    if entries.is_empty() {
        log::warn!(
            "Category remap file {} contains no [[remap]] entries — no categories will be remapped",
            path.display()
        );
    } else {
        log::info!(
            "Loaded {} category remap entries from {}",
            entries.len(),
            path.display()
        );
    }

    Ok(entries)
}

/// Rewrites each transaction's `category` to the configured replacement when
/// its (type, category) exactly matches a remap entry's (type, from) —
/// after trimming surrounding whitespace from the transaction's category.
/// Matching is otherwise an exact, case-sensitive string comparison. Logs an
/// info line for each transaction actually rewritten.
pub fn apply(transactions: &mut [Transaction], entries: &[RemapEntry]) {
    for transaction in transactions {
        let category = transaction.category.trim();
        if let Some(entry) = entries.iter().find(|entry| {
            entry.transaction_type == transaction.transaction_type && entry.from == category
        }) {
            log::info!(
                "Remapped category \"{}\" -> \"{}\" for {:?} transaction on {}",
                entry.from,
                entry.to,
                transaction.transaction_type,
                transaction.date
            );
            transaction.category = entry.to.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn temp_toml_path(test_name: &str) -> std::path::PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "moneylens-category-remap-{test_name}-{unique_suffix}.toml"
        ))
    }

    fn write_temp_toml(test_name: &str, contents: &str) -> std::path::PathBuf {
        let path = temp_toml_path(test_name);
        std::fs::write(&path, contents).expect("writing temp remap file should succeed");
        path
    }

    #[test]
    fn loads_remap_entries_from_toml() {
        let path = write_temp_toml(
            "loads-entries",
            r#"
            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Other"
            "#,
        );

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
    fn trims_whitespace_from_from_and_to() {
        let path = write_temp_toml(
            "trims-whitespace",
            r#"
            [[remap]]
            type = "spend"
            from = "  Utilities  "
            to = "  Utilities/Other  "
            "#,
        );

        let entries = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(entries[0].from, "Utilities");
        assert_eq!(entries[0].to, "Utilities/Other");
    }

    #[test]
    fn missing_file_is_an_error() {
        let result = load(&std::path::PathBuf::from(
            "/nonexistent/moneylens-category-remap.toml",
        ));

        let err = result.expect_err("missing file should fail to load");
        assert!(
            err.to_string()
                .contains("/nonexistent/moneylens-category-remap.toml")
        );
    }

    #[test]
    fn malformed_toml_is_an_error() {
        let path = write_temp_toml("malformed", "this is not valid toml [[[");

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn unknown_table_name_is_an_error_not_an_empty_list() {
        // A typo'd table name ("remaps" instead of "remap") must not
        // silently parse as zero entries.
        let path = write_temp_toml(
            "unknown-table",
            r#"
            [[remaps]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Other"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn blank_from_is_an_error() {
        let path = write_temp_toml(
            "blank-from",
            r#"
            [[remap]]
            type = "spend"
            from = "   "
            to = "Utilities/Other"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn blank_to_is_an_error() {
        let path = write_temp_toml(
            "blank-to",
            r#"
            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "   "
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn from_equal_to_is_an_error() {
        let path = write_temp_toml(
            "noop-entry",
            r#"
            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "Utilities"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn duplicate_type_and_from_is_an_error() {
        let path = write_temp_toml(
            "duplicate-entry",
            r#"
            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Electricity"

            [[remap]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Water"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn same_from_under_different_types_is_allowed() {
        let path = write_temp_toml(
            "same-from-different-type",
            r#"
            [[remap]]
            type = "spend"
            from = "Other"
            to = "Other/Misc"

            [[remap]]
            type = "earn"
            from = "Other"
            to = "Other/Bonus"
            "#,
        );

        let entries = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(entries.len(), 2);
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
    fn matches_after_trimming_transaction_category_whitespace() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let mut transactions = vec![transaction(TransactionType::Spend, "  Utilities  ")];
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

    #[test]
    fn applies_independently_across_multiple_transactions() {
        let entries = vec![
            RemapEntry {
                transaction_type: TransactionType::Spend,
                from: "Utilities".to_string(),
                to: "Utilities/Other".to_string(),
            },
            RemapEntry {
                transaction_type: TransactionType::Spend,
                from: "Transport".to_string(),
                to: "Transport/Other".to_string(),
            },
        ];

        let mut transactions = vec![
            transaction(TransactionType::Spend, "Utilities"),
            transaction(TransactionType::Spend, "Groceries"),
            transaction(TransactionType::Earn, "Utilities"),
            transaction(TransactionType::Spend, "Transport"),
        ];
        apply(&mut transactions, &entries);

        assert_eq!(transactions[0].category, "Utilities/Other");
        assert_eq!(transactions[1].category, "Groceries");
        assert_eq!(transactions[2].category, "Utilities");
        assert_eq!(transactions[3].category, "Transport/Other");
    }
}
