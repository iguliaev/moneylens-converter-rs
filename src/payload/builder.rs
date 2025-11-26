use super::types::Payload;

pub struct PayloadBuilder {
    payload: Payload,
}

impl PayloadBuilder {
    pub fn new() -> Self {
        Self {
            payload: Payload::default(),
        }
    }

    pub fn add_transactions(mut self, transactions: Vec<super::types::Transaction>) -> Self {
        self.payload.transactions.extend(transactions);
        self
    }

    pub fn build(self) -> Payload {
        self.payload
    }
}
