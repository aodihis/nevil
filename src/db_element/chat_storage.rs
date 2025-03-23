use crate::db_element::chat::Message;
use bincode::config;
use sled::Db;
use uuid::Uuid;

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

    pub fn add_message(&self, db_uuid: &Uuid, message: &Message) -> Result<(), String> {
        let tree = self.db.open_tree("messages").expect("Unable to open messages");
        let key = format!("{}:{:?}:{}", db_uuid, message.timestamp.timestamp_micros(), message.uuid);

        let config = config::standard();
        let encode = bincode::encode_to_vec(message, config).unwrap();
        // let serialized = standard().ser
        tree.insert(&key.as_bytes(), encode).expect("Failed to save message");
        Ok(())
    }
}