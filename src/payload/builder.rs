use super::types::Payload;

#[derive(Default)]
pub struct PayloadBuilder {
    payload: Payload,
    category_set:
        std::collections::HashSet<(super::types::TransactionType, Option<String>, String)>,
    bank_account_set: std::collections::HashSet<String>,
    tag_set: std::collections::HashSet<String>,
}

/// Splits a transaction's `"Parent/Child"` category string on the first `/`
/// (trimming both sides), mirroring the API's own rule for
/// `TransactionInput.category`. Bare names (no `/`) return `(None, name)`.
fn split_category(category: &str) -> (Option<String>, String) {
    match category.split_once('/') {
        Some((parent, child)) => (Some(parent.trim().to_string()), child.trim().to_string()),
        None => (None, category.trim().to_string()),
    }
}

impl PayloadBuilder {
    pub fn add_transactions(mut self, transactions: Vec<super::types::Transaction>) -> Self {
        transactions.iter().for_each(|tx| {
            // Add unique categories, splitting "Parent/Child" into a root
            // parent entry + a child entry with `parent` set, matching the
            // API's CategoryInput.parent field.
            let (parent, name) = split_category(&tx.category);

            if let Some(ref parent_name) = parent {
                let parent_key = (tx.type_.clone(), None, parent_name.clone());
                if self.category_set.insert(parent_key) {
                    self.payload.categories.push(super::types::Category {
                        name: parent_name.clone(),
                        type_: tx.type_.clone(),
                        description: None,
                        parent: None,
                    });
                }
            }

            let child_key = (tx.type_.clone(), parent.clone(), name.clone());
            if self.category_set.insert(child_key) {
                self.payload.categories.push(super::types::Category {
                    name,
                    type_: tx.type_.clone(),
                    description: None,
                    parent,
                });
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
        });

        self.payload.transactions.extend(transactions);
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
}
