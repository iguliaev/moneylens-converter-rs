use super::types::Payload;

pub struct PayloadBuilder {
    payload: Payload,
}

impl PayloadBuilder {
    pub fn new() -> Self {
        Self {
            payload: Payload::default()
        }
    }
}
