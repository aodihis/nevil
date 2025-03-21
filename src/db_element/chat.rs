use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Sender {
    System,
    User
}
pub struct Message {
    pub uuid: Uuid,
    pub sender: Sender,
    pub content: String,
    pub is_sql: bool,
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

pub struct Chat {
    pub uuid: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Chat {
    pub fn new(uuid: Uuid) -> Self {
        let now = Utc::now();
        Self {
            uuid,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now();
    }
}