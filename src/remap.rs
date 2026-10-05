use crate::payload::builder::{CategorySplit, split_category};
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

/// A bank account remap entry. Unlike `RemapEntry`, this has no `type` —
/// a bank account isn't scoped by transaction type in the data model, so
/// one entry applies regardless of whether the account is used on a
/// spend/save/earn transaction.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BankAccountRemapEntry {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct RemapFile {
    #[serde(default)]
    remap: RemapSection,
}

/// One sub-table per remappable entity. `category` and `bank_account`
/// exist today; a future entity (e.g. `tag`) would be added here as a
/// sibling, its own `[[remap.<entity>]]` array.
#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct RemapSection {
    #[serde(default)]
    category: Vec<RemapEntry>,
    #[serde(default)]
    bank_account: Vec<BankAccountRemapEntry>,
}

/// The validated, loaded contents of a `--remap` file, one list per entity.
#[derive(Debug, Default)]
pub struct RemapConfig {
    pub category: Vec<RemapEntry>,
    pub bank_account: Vec<BankAccountRemapEntry>,
}

#[derive(Debug)]
struct InvalidRemapEntry {
    description: String,
}

impl fmt::Display for InvalidRemapEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid remap entry: {}", self.description)
    }
}

impl Error for InvalidRemapEntry {}

fn validate_category_entries(entries: Vec<RemapEntry>) -> Result<Vec<RemapEntry>, Box<dyn Error>> {
    let entries: Vec<RemapEntry> = entries
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
                    "category \"from\" is blank for type \"{}\"",
                    entry.transaction_type
                ),
            }));
        }
        if entry.to.is_empty() {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "category \"to\" is blank for type \"{}\", from \"{}\"",
                    entry.transaction_type, entry.from
                ),
            }));
        }
        if entry.from == entry.to {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "category \"from\" and \"to\" are both \"{}\" for type \"{}\" — this entry is a no-op",
                    entry.from, entry.transaction_type
                ),
            }));
        }
        match split_category(&entry.to) {
            CategorySplit::Multipath => {
                return Err(Box::new(InvalidRemapEntry {
                    description: format!(
                        "category \"to\" \"{}\" for type \"{}\", from \"{}\" has more than one level of nesting — categories support at most one parent/child level",
                        entry.to, entry.transaction_type, entry.from
                    ),
                }));
            }
            CategorySplit::Empty => {
                return Err(Box::new(InvalidRemapEntry {
                    description: format!(
                        "category \"to\" for type \"{}\", from \"{}\" has no usable category name after trimming each side of \"/\"",
                        entry.transaction_type, entry.from
                    ),
                }));
            }
            CategorySplit::Root(_) | CategorySplit::Nested { .. } => {}
        }
        if !seen.insert((&entry.transaction_type, &entry.from)) {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "duplicate category entry for type \"{}\", from \"{}\"",
                    entry.transaction_type, entry.from
                ),
            }));
        }
    }

    Ok(entries)
}

fn validate_bank_account_entries(
    entries: Vec<BankAccountRemapEntry>,
) -> Result<Vec<BankAccountRemapEntry>, Box<dyn Error>> {
    let entries: Vec<BankAccountRemapEntry> = entries
        .into_iter()
        .map(|entry| BankAccountRemapEntry {
            from: entry.from.trim().to_string(),
            to: entry.to.trim().to_string(),
        })
        .collect();

    let mut seen = HashSet::new();
    for entry in &entries {
        if entry.from.is_empty() {
            return Err(Box::new(InvalidRemapEntry {
                description: "bank_account \"from\" is blank".to_string(),
            }));
        }
        if entry.to.is_empty() {
            return Err(Box::new(InvalidRemapEntry {
                description: format!("bank_account \"to\" is blank, from \"{}\"", entry.from),
            }));
        }
        if entry.from == entry.to {
            return Err(Box::new(InvalidRemapEntry {
                description: format!(
                    "bank_account \"from\" and \"to\" are both \"{}\" — this entry is a no-op",
                    entry.from
                ),
            }));
        }
        if !seen.insert(&entry.from) {
            return Err(Box::new(InvalidRemapEntry {
                description: format!("duplicate bank_account entry for from \"{}\"", entry.from),
            }));
        }
    }

    Ok(entries)
}

/// Loads a `--remap` TOML file: `[[remap.category]]` and
/// `[[remap.bank_account]]` arrays of tables.
///
/// `[[remap.category]]` entries have a `type` (`spend`/`save`/`earn`),
/// `from`, and `to` category name — either a bare leaf, or
/// `"Parent/Child"` for one level of nesting, e.g.:
///
/// ```toml
/// [[remap.category]]
/// type = "spend"
/// from = "Utilities"
/// to = "Utilities/Other"
/// ```
///
/// Rejects (as an error naming the offending entry) a blank `from`/`to`, a
/// `to` that isn't a plain leaf name or a single `"Parent/Child"` level
/// (anything `PayloadBuilder` would otherwise reject downstream as
/// `Multipath`/`Empty`), `from == to`, and two entries sharing the same
/// `(type, from)` — all of these would otherwise be silently useless,
/// ambiguous, or only discovered later as a confusing upload error.
///
/// `[[remap.bank_account]]` entries have just a `from`/`to` (no `type` —
/// bank accounts aren't scoped by transaction type), e.g.:
///
/// ```toml
/// [[remap.bank_account]]
/// from = "X"
/// to = "AmEx"
/// ```
///
/// Validated the same way (blank `from`/`to`, `from == to`, duplicate
/// `from`), minus the category-specific nesting-depth check, since bank
/// account names have no parent/child structure. `spend.rs`'s bank account
/// symbol resolution uses the literal string `"(empty)"` as `from` to mean
/// "the spreadsheet cell had no symbol at all" — this isn't a blank string,
/// so it needs no special handling here; it's just another valid `from`.
///
/// An unrecognized key anywhere in the file (e.g. a typo'd
/// `[[remap.categories]]` table) is also rejected rather than quietly
/// parsing as zero entries.
pub fn load(path: &Path) -> Result<RemapConfig, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read remap file {}: {e}", path.display()))?;

    let file: RemapFile = toml::from_str(&contents)
        .map_err(|e| format!("failed to parse remap file {}: {e}", path.display()))?;

    let category = validate_category_entries(file.remap.category)?;
    let bank_account = validate_bank_account_entries(file.remap.bank_account)?;

    if category.is_empty() && bank_account.is_empty() {
        log::warn!(
            "Remap file {} contains no [[remap.category]] or [[remap.bank_account]] entries — nothing will be remapped",
            path.display()
        );
    } else {
        log::info!(
            "Loaded {} category and {} bank_account remap entries from {}",
            category.len(),
            bank_account.len(),
            path.display()
        );
    }

    Ok(RemapConfig {
        category,
        bank_account,
    })
}

/// Maps each transaction to one with its `category` rewritten to the
/// configured replacement when its (type, category) exactly matches a
/// remap entry's (type, from) — after trimming surrounding whitespace from
/// the transaction's category. Matching is otherwise an exact,
/// case-sensitive string comparison. Logs an info line for each transaction
/// actually rewritten.
pub fn apply_categories(
    transactions: Vec<Transaction>,
    entries: &[RemapEntry],
) -> Vec<Transaction> {
    transactions
        .into_iter()
        .map(|mut transaction| {
            let category = transaction.category.trim();
            if let Some(entry) = entries.iter().find(|entry| {
                entry.transaction_type == transaction.transaction_type && entry.from == category
            }) {
                log::info!(
                    "Remapped category \"{}\" -> \"{}\" for {} transaction on {}",
                    entry.from,
                    entry.to,
                    transaction.transaction_type,
                    transaction.date
                );
                transaction.category = entry.to.clone();
            }
            transaction
        })
        .collect()
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
        std::env::temp_dir().join(format!("moneylens-remap-{test_name}-{unique_suffix}.toml"))
    }

    fn write_temp_toml(test_name: &str, contents: &str) -> std::path::PathBuf {
        let path = temp_toml_path(test_name);
        std::fs::write(&path, contents).expect("writing temp remap file should succeed");
        path
    }

    #[test]
    fn loads_category_entries_from_toml() {
        let path = write_temp_toml(
            "loads-category-entries",
            r#"
            [[remap.category]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Other"
            "#,
        );

        let config = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(
            config.category,
            vec![RemapEntry {
                transaction_type: TransactionType::Spend,
                from: "Utilities".to_string(),
                to: "Utilities/Other".to_string(),
            }]
        );
        assert!(config.bank_account.is_empty());
    }

    #[test]
    fn loads_bank_account_entries_from_toml() {
        let path = write_temp_toml(
            "loads-bank-account-entries",
            r#"
            [[remap.bank_account]]
            from = "X"
            to = "AmEx"

            [[remap.bank_account]]
            from = "(empty)"
            to = "Monzo"
            "#,
        );

        let config = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(
            config.bank_account,
            vec![
                BankAccountRemapEntry {
                    from: "X".to_string(),
                    to: "AmEx".to_string(),
                },
                BankAccountRemapEntry {
                    from: "(empty)".to_string(),
                    to: "Monzo".to_string(),
                },
            ]
        );
        assert!(config.category.is_empty());
    }

    #[test]
    fn trims_whitespace_from_from_and_to() {
        let path = write_temp_toml(
            "trims-whitespace",
            r#"
            [[remap.category]]
            type = "spend"
            from = "  Utilities  "
            to = "  Utilities/Other  "
            "#,
        );

        let config = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(config.category[0].from, "Utilities");
        assert_eq!(config.category[0].to, "Utilities/Other");
    }

    #[test]
    fn missing_file_is_an_error() {
        let result = load(&std::path::PathBuf::from(
            "/nonexistent/moneylens-remap.toml",
        ));

        let err = result.expect_err("missing file should fail to load");
        assert!(
            err.to_string()
                .contains("/nonexistent/moneylens-remap.toml")
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
        // A typo'd top-level table name ("remaps" instead of "remap") must
        // not silently parse as zero entries.
        let path = write_temp_toml(
            "unknown-table",
            r#"
            [[remaps.category]]
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
    fn unknown_entity_name_is_an_error_not_an_empty_list() {
        // A typo'd entity name ("categories" instead of "category") under
        // the correct top-level "remap" table must not silently parse as
        // zero entries either.
        let path = write_temp_toml(
            "unknown-entity",
            r#"
            [[remap.categories]]
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
    fn blank_category_from_is_an_error() {
        let path = write_temp_toml(
            "blank-category-from",
            r#"
            [[remap.category]]
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
    fn blank_category_to_is_an_error() {
        let path = write_temp_toml(
            "blank-category-to",
            r#"
            [[remap.category]]
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
    fn category_from_equal_to_is_an_error() {
        let path = write_temp_toml(
            "noop-category-entry",
            r#"
            [[remap.category]]
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
    fn category_to_with_more_than_one_nesting_level_is_an_error() {
        let path = write_temp_toml(
            "multipath-to",
            r#"
            [[remap.category]]
            type = "spend"
            from = "Utilities"
            to = "Bills/Utilities/Electricity"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn category_to_that_is_only_a_slash_is_an_error() {
        let path = write_temp_toml(
            "empty-to-slash",
            r#"
            [[remap.category]]
            type = "spend"
            from = "Utilities"
            to = "/"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn category_to_with_exactly_one_nesting_level_is_allowed() {
        let path = write_temp_toml(
            "nested-to",
            r#"
            [[remap.category]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Other"
            "#,
        );

        let config = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(config.category[0].to, "Utilities/Other");
    }

    #[test]
    fn duplicate_category_type_and_from_is_an_error() {
        let path = write_temp_toml(
            "duplicate-category-entry",
            r#"
            [[remap.category]]
            type = "spend"
            from = "Utilities"
            to = "Utilities/Electricity"

            [[remap.category]]
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
    fn same_category_from_under_different_types_is_allowed() {
        let path = write_temp_toml(
            "same-from-different-type",
            r#"
            [[remap.category]]
            type = "spend"
            from = "Other"
            to = "Other/Misc"

            [[remap.category]]
            type = "earn"
            from = "Other"
            to = "Other/Bonus"
            "#,
        );

        let config = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(config.category.len(), 2);
    }

    #[test]
    fn blank_bank_account_from_is_an_error() {
        let path = write_temp_toml(
            "blank-bank-account-from",
            r#"
            [[remap.bank_account]]
            from = "   "
            to = "AmEx"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn blank_bank_account_to_is_an_error() {
        let path = write_temp_toml(
            "blank-bank-account-to",
            r#"
            [[remap.bank_account]]
            from = "X"
            to = "   "
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn bank_account_from_equal_to_is_an_error() {
        let path = write_temp_toml(
            "noop-bank-account-entry",
            r#"
            [[remap.bank_account]]
            from = "AmEx"
            to = "AmEx"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn duplicate_bank_account_from_is_an_error() {
        let path = write_temp_toml(
            "duplicate-bank-account-entry",
            r#"
            [[remap.bank_account]]
            from = "X"
            to = "AmEx"

            [[remap.bank_account]]
            from = "X"
            to = "Amex Platinum"
            "#,
        );

        let result = load(&path);
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert!(result.is_err());
    }

    #[test]
    fn bank_account_empty_sentinel_is_a_valid_from() {
        let path = write_temp_toml(
            "empty-sentinel",
            r#"
            [[remap.bank_account]]
            from = "(empty)"
            to = "Monzo"
            "#,
        );

        let config = load(&path).expect("remap file should parse");
        std::fs::remove_file(&path).expect("temp remap file should be removed");

        assert_eq!(config.bank_account[0].from, "(empty)");
    }

    #[test]
    fn rewrites_category_on_exact_type_and_name_match() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let transactions = vec![transaction(TransactionType::Spend, "Utilities")];
        let transactions = apply_categories(transactions, &entries);

        assert_eq!(transactions[0].category, "Utilities/Other");
    }

    #[test]
    fn matches_after_trimming_transaction_category_whitespace() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let transactions = vec![transaction(TransactionType::Spend, "  Utilities  ")];
        let transactions = apply_categories(transactions, &entries);

        assert_eq!(transactions[0].category, "Utilities/Other");
    }

    #[test]
    fn leaves_non_matching_categories_untouched() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let transactions = vec![transaction(TransactionType::Spend, "Groceries")];
        let transactions = apply_categories(transactions, &entries);

        assert_eq!(transactions[0].category, "Groceries");
    }

    #[test]
    fn does_not_remap_across_different_transaction_types() {
        let entries = vec![RemapEntry {
            transaction_type: TransactionType::Spend,
            from: "Utilities".to_string(),
            to: "Utilities/Other".to_string(),
        }];

        let transactions = vec![transaction(TransactionType::Earn, "Utilities")];
        let transactions = apply_categories(transactions, &entries);

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

        let transactions = vec![
            transaction(TransactionType::Spend, "Utilities"),
            transaction(TransactionType::Spend, "Groceries"),
            transaction(TransactionType::Earn, "Utilities"),
            transaction(TransactionType::Spend, "Transport"),
        ];
        let transactions = apply_categories(transactions, &entries);

        assert_eq!(transactions[0].category, "Utilities/Other");
        assert_eq!(transactions[1].category, "Groceries");
        assert_eq!(transactions[2].category, "Utilities");
        assert_eq!(transactions[3].category, "Transport/Other");
    }
}
