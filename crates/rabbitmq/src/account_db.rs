use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    AccountUpdate {
        key: [u8; 32],
        owner: [u8; 32],
        data: Vec<u8>,
    },
}
