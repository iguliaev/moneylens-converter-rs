use super::types::Payload;

#[derive(Default)]
pub struct PayloadBuilder {
    payload: Payload,
    category_set: std::collections::HashSet<super::types::Category>,
    bank_account_set: std::collections::HashSet<String>,
    tag_set: std::collections::HashSet<String>,
}

/// The result of splitting a transaction's category string, mirroring the
/// API's own `"Parent/Child"` rule for `TransactionInput.category` (split on
/// the first `/`, each side trimmed).
enum CategorySplit {
    /// No `/`, or a `/` split where one side trims down to nothing (e.g.
    /// `"Bills"`, `"Bills/  "`, or `"  /Bills"`) — a plain root-level
    /// category using whichever side is non-empty.
    Root(String),
    /// A `"Parent/Child"` path with exactly one `/` and non-empty content
    /// on both sides.
    Nested { parent: String, name: String },
    /// More than one `/` (e.g. `"A/B/C"`) — the schema caps hierarchy at 2
    /// levels, so this can't be represented. The caller logs a warning and
    /// skips adding any `categories[]` entries for it, leaving the
    /// transaction's own `category` field untouched.
    Multipath,
    /// Both sides trim down to nothing (e.g. `"/"`, `"   "`, `" / "`) — no
    /// usable category name at all. The caller logs a warning and skips it,
    /// leaving the transaction's own `category` field untouched.
    Empty,
}

fn split_category(category: &str) -> CategorySplit {
    let Some((parent, child)) = category.split_once('/') else {
        let name = category.trim().to_string();
        return if name.is_empty() {
            CategorySplit::Empty
        } else {
            CategorySplit::Root(name)
        };
    };

    let parent = parent.trim().to_string();
    let child = child.trim().to_string();

    match (parent.is_empty(), child.is_empty()) {
        (true, true) => CategorySplit::Empty,
        (true, false) => CategorySplit::Root(child),
        (false, true) => CategorySplit::Root(parent),
        (false, false) if child.contains('/') => CategorySplit::Multipath,
        (false, false) => CategorySplit::Nested { parent, name: child },
    }
}

impl PayloadBuilder {
    pub fn add_transactions(mut self, transactions: Vec<super::types::Transaction>) -> Self {
        for mut tx in transactions {
            // Add unique categories, splitting "Parent/Child" into a root
            // parent entry + a child entry with `parent` set, matching the
            // API's CategoryInput.parent field. The transaction's own
            // `category` is normalized to the same trimmed form so it's
            // guaranteed to match the categories[] entries emitted here,
            // even if the source cell had incidental whitespace.
            match split_category(&tx.category) {
                CategorySplit::Root(name) => {
                    let category = super::types::Category {
                        name: name.clone(),
                        type_: tx.type_.clone(),
                        description: None,
                        parent: None,
                    };
                    if self.category_set.insert(category.clone()) {
                        self.payload.categories.push(category);
                    }
                    tx.category = name;
                }
                CategorySplit::Nested { parent, name } => {
                    let parent_category = super::types::Category {
                        name: parent.clone(),
                        type_: tx.type_.clone(),
                        description: None,
                        parent: None,
                    };
                    if self.category_set.insert(parent_category.clone()) {
                        self.payload.categories.push(parent_category);
                    }

                    let child_category = super::types::Category {
                        name: name.clone(),
                        type_: tx.type_.clone(),
                        description: None,
                        parent: Some(parent.clone()),
                    };
                    if self.category_set.insert(child_category.clone()) {
                        self.payload.categories.push(child_category);
                    }

                    tx.category = format!("{parent}/{name}");
                }
                CategorySplit::Multipath => {
                    eprintln!(
                        "Warning: category \"{}\" has more than one level of nesting; \
                         multi-level category paths are not supported, skipping category \
                         entry for this transaction",
                        tx.category
                    );
                }
                CategorySplit::Empty => {
                    eprintln!(
                        "Warning: category \"{}\" has no usable name after trimming; \
                         skipping category entry for this transaction",
                        tx.category
                    );
                }
            }

            // Add unique bank accounts
            if !self.bank_account_set.contains(&tx.bank_account) {
                self.payload.bank_accounts.push(super::types::BankAccount {
                    name: tx.bank_account.clone(),
                    description: None,
                });
                self.bank_account_set.insert(tx.bank_account.clone());
            }

            // Add unique tags
            for tag in &tx.tags {
                if !self.tag_set.contains(tag) {
                    self.payload.tags.push(super::types::Tag {
                        name: tag.clone(),
                        description: None,
                    });
                    self.tag_set.insert(tag.clone());
                }
            }

            self.payload.transactions.push(tx);
        }

        self
    }

    pub fn build(self) -> Payload {
        self.payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::types::{Transaction, TransactionType};

    #[test]
    fn test_payload_builder() {
        let transactions = vec![
            Transaction {
                date: "2024-01-01".to_string(),
                type_: TransactionType::Spend,
                category: "Food".to_string(),
                bank_account: "Checking".to_string(),
                amount: 50.0,
                tags: vec!["Groceries".to_string()],
                notes: Some("Weekly groceries".to_string()),
            },
            Transaction {
                date: "2024-01-02".to_string(),
                type_: TransactionType::Earn,
                category: "Salary".to_string(),
                bank_account: "Checking".to_string(),
                amount: 2000.0,
                tags: vec![],
                notes: Some("Monthly salary".to_string()),
            },
        ];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions.clone())
            .add_transactions(transactions)
            .build();

        assert_eq!(
            payload.transactions.len(),
            4,
            "Expected 4 transactions after adding the same transactions twice"
        );
        assert_eq!(payload.categories.len(), 2, "Expected 2 unique categories");
        assert_eq!(
            payload.bank_accounts.len(),
            1,
            "Expected 1 unique bank account"
        );
        assert_eq!(payload.tags.len(), 1, "Expected 1 unique tag");
    }

    #[test]
    fn splits_slash_separated_category_into_parent_and_child_entries() {
        let transactions = vec![Transaction {
            date: "2025-01-10".to_string(),
            type_: TransactionType::Spend,
            category: "Vacation/Accomodation".to_string(),
            bank_account: "AmEx".to_string(),
            amount: 100.0,
            tags: vec![],
            notes: None,
        }];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions.clone())
            .add_transactions(transactions) // re-adding must not duplicate entries
            .build();

        assert_eq!(payload.transactions.len(), 2);
        assert_eq!(
            payload.categories.len(),
            2,
            "expected exactly one parent entry and one child entry, deduped across both adds"
        );

        let parent = payload
            .categories
            .iter()
            .find(|c| c.name == "Vacation")
            .expect("parent category entry should exist");
        assert_eq!(parent.parent, None);
        assert_eq!(parent.type_, TransactionType::Spend);

        let child = payload
            .categories
            .iter()
            .find(|c| c.name == "Accomodation")
            .expect("child category entry should exist");
        assert_eq!(child.parent, Some("Vacation".to_string()));
        assert_eq!(child.type_, TransactionType::Spend);

        // the transaction's own `category` field must remain the full path, unchanged
        assert!(
            payload
                .transactions
                .iter()
                .all(|tx| tx.category == "Vacation/Accomodation")
        );
    }

    #[test]
    fn same_bare_category_name_under_different_types_gets_separate_entries() {
        let transactions = vec![
            Transaction {
                date: "2025-01-01".to_string(),
                type_: TransactionType::Save,
                category: "Other".to_string(),
                bank_account: "Default Account".to_string(),
                amount: 10.0,
                tags: vec![],
                notes: None,
            },
            Transaction {
                date: "2025-01-02".to_string(),
                type_: TransactionType::Earn,
                category: "Other".to_string(),
                bank_account: "Default Account".to_string(),
                amount: 20.0,
                tags: vec![],
                notes: None,
            },
        ];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions)
            .build();

        assert_eq!(
            payload.categories.len(),
            2,
            "same bare name under two different transaction types must produce two category entries"
        );
        assert!(
            payload
                .categories
                .iter()
                .any(|c| c.name == "Other" && c.type_ == TransactionType::Save)
        );
        assert!(
            payload
                .categories
                .iter()
                .any(|c| c.name == "Other" && c.type_ == TransactionType::Earn)
        );
    }

    #[test]
    fn trailing_slash_with_no_child_collapses_to_a_root_category() {
        let transactions = vec![
            Transaction {
                date: "2025-01-10".to_string(),
                type_: TransactionType::Spend,
                category: "Bills/".to_string(),
                bank_account: "AmEx".to_string(),
                amount: 10.0,
                tags: vec![],
                notes: None,
            },
            Transaction {
                date: "2025-01-11".to_string(),
                type_: TransactionType::Spend,
                category: "Food/   ".to_string(),
                bank_account: "AmEx".to_string(),
                amount: 20.0,
                tags: vec![],
                notes: None,
            },
        ];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions)
            .build();

        assert_eq!(payload.categories.len(), 2);
        let bills = payload
            .categories
            .iter()
            .find(|c| c.name == "Bills")
            .expect("\"Bills/\" should collapse to a root category named \"Bills\"");
        assert_eq!(bills.parent, None);

        let food = payload
            .categories
            .iter()
            .find(|c| c.name == "Food")
            .expect("\"Food/   \" should collapse to a root category named \"Food\"");
        assert_eq!(food.parent, None);

        // the transactions' own `category` fields are normalized to match
        assert!(
            payload
                .transactions
                .iter()
                .any(|tx| tx.category == "Bills")
        );
        assert!(payload.transactions.iter().any(|tx| tx.category == "Food"));
    }

    #[test]
    fn leading_slash_with_no_parent_collapses_to_a_root_category() {
        let transactions = vec![Transaction {
            date: "2025-01-10".to_string(),
            type_: TransactionType::Spend,
            category: "  /Child".to_string(),
            bank_account: "AmEx".to_string(),
            amount: 10.0,
            tags: vec![],
            notes: None,
        }];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions)
            .build();

        assert_eq!(payload.categories.len(), 1);
        let child = &payload.categories[0];
        assert_eq!(child.name, "Child");
        assert_eq!(child.parent, None);

        assert_eq!(payload.transactions[0].category, "Child");
    }

    #[test]
    fn blank_category_produces_no_entries_and_leaves_transaction_untouched() {
        let transactions = vec![
            Transaction {
                date: "2025-01-10".to_string(),
                type_: TransactionType::Spend,
                category: "   ".to_string(),
                bank_account: "AmEx".to_string(),
                amount: 10.0,
                tags: vec![],
                notes: None,
            },
            Transaction {
                date: "2025-01-11".to_string(),
                type_: TransactionType::Spend,
                category: " / ".to_string(),
                bank_account: "AmEx".to_string(),
                amount: 20.0,
                tags: vec![],
                notes: None,
            },
        ];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions)
            .build();

        assert!(
            payload.categories.is_empty(),
            "a category with no usable name on either side must not produce any category entries"
        );
        assert_eq!(payload.transactions.len(), 2);
        assert_eq!(payload.transactions[0].category, "   ");
        assert_eq!(payload.transactions[1].category, " / ");
    }

    #[test]
    fn multipath_category_is_skipped_with_a_warning() {
        let transactions = vec![Transaction {
            date: "2025-01-10".to_string(),
            type_: TransactionType::Spend,
            category: "Travel/Europe/Hotels".to_string(),
            bank_account: "AmEx".to_string(),
            amount: 30.0,
            tags: vec![],
            notes: None,
        }];

        let payload = PayloadBuilder::default()
            .add_transactions(transactions)
            .build();

        assert!(
            payload.categories.is_empty(),
            "a category path with more than one level of nesting must not produce any category entries"
        );
        // the transaction itself is still emitted unchanged; the API will
        // reject it at upload time since the referenced category doesn't exist
        assert_eq!(payload.transactions.len(), 1);
        assert_eq!(payload.transactions[0].category, "Travel/Europe/Hotels");
    }
}
