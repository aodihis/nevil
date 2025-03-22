use bincode::Encode;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Encode)]
#[serde(rename_all = "lowercase")]
pub enum Sender {
    System,
    User
}

#[derive(Encode)]
pub struct Message {
    #[bincode(with_serde)]
    pub uuid: Uuid,
    pub sender: Sender,
    pub content: String,
    pub is_sql: bool,
    #[bincode(with_serde)]
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(sender: Sender, content: String, is_sql: bool) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            sender,
            content,
            is_sql,
            timestamp: Utc::now(),
        }
    }
}