use super::types::Payload;

#[derive(Default)]
pub struct PayloadBuilder {
    payload: Payload,
}

impl PayloadBuilder {
    pub fn add_transactions(mut self, transactions: Vec<super::types::Transaction>) -> Self {
        self.payload.transactions.extend(transactions);
        self
    }

    pub fn build(self) -> Payload {
        self.payload
    }
}
