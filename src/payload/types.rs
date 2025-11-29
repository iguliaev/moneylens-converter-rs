use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum TransactionType {
    Spend,
    Save,
    Earn,
}

impl TransactionType {
    pub fn as_str(&self) -> &str {
        match self {
            TransactionType::Spend => "spend",
            TransactionType::Save => "save",
            TransactionType::Earn => "earn",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: TransactionType,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BankAccount {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Payload {
    pub categories: Vec<Category>,
    pub bank_accounts: Vec<BankAccount>,
    pub tags: Vec<Tag>,
    pub transactions: Vec<Transaction>,
}
