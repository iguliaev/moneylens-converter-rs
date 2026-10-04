use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Spend,
    Save,
    Earn,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TransactionType::Spend => "spend",
            TransactionType::Save => "save",
            TransactionType::Earn => "earn",
        };
        write!(f, "{s}")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub name: String,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
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
    pub transaction_type: TransactionType,
    pub category: String,
    pub bank_account: String,
    pub amount: f64,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Payload {
    pub categories: Vec<Category>,
    pub bank_accounts: Vec<BankAccount>,
    pub tags: Vec<Tag>,
    pub transactions: Vec<Transaction>,
}
