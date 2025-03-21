use std::sync::Arc;
use sled::Db;
use tokio::sync::Mutex;
use crate::db_element::chat::Chat;

pub struct ChatStorage {
    db: Db,
}

impl ChatStorage {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = sled::open(db_path).expect("Unable to open sled database");

        Ok(Self {
            db,
        })
    }
}