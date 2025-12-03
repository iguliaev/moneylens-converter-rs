use super::types::Payload;

#[derive(Default)]
pub struct PayloadBuilder {
    payload: Payload,
    category_set: std::collections::HashSet<String>,
    bank_account_set: std::collections::HashSet<String>,
    tag_set: std::collections::HashSet<String>,
}

impl PayloadBuilder {
    pub fn add_transactions(&mut self, transactions: Vec<super::types::Transaction>) -> &mut Self {
        transactions.iter().for_each(|tx| {
            // Add unique categories
            if !self.category_set.contains(&tx.category) {
                self.payload.categories.push(super::types::Category {
                    name: tx.category.clone(),
                    type_: tx.type_.clone(),
                    description: None,
                });
                self.category_set.insert(tx.category.clone());
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
            if let Some(tags) = &tx.tags {
                for tag in tags {
                    if !self.tag_set.contains(tag) {
                        self.payload.tags.push(super::types::Tag {
                            name: tag.clone(),
                            description: None,
                        });
                        self.tag_set.insert(tag.clone());
                    }
                }
            }
        });

        self.payload.transactions.extend(transactions);
        self
    }

    pub fn build(&self) -> Payload {
        self.payload.clone()
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
                tags: Some(vec!["Groceries".to_string()]),
                notes: Some("Weekly groceries".to_string()),
            },
            Transaction {
                date: "2024-01-02".to_string(),
                type_: TransactionType::Earn,
                category: "Salary".to_string(),
                bank_account: "Checking".to_string(),
                amount: 2000.0,
                tags: None,
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
}
