use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Spend,
    Save,
    Earn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BankAccount {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub date: String,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub category: String,
    pub bank_account: String,
    pub amount: f64,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Payload {
    pub categories: Vec<Category>,
    pub bank_accounts: Vec<BankAccount>,
    pub tags: Vec<Tag>,
    pub transactions: Vec<Transaction>,
}
