use std::path::PathBuf;
use crate::db_element::chat::Message;
use bincode::config;
use log::debug;
use sled::Db;
use uuid::Uuid;

pub struct ChatStorage {
    db: Db,
}

impl ChatStorage {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let db = sled::open(db_path).expect("Unable to open sled database");

        Ok(Self {
            db,
        })
    }

    pub fn add_message(&self, conversation_uuid: &Uuid, message: &Message) -> Result<(), String> {
        let tree = self.db.open_tree("messages").expect("Unable to open messages");
        let key = format!("{}:{:?}:{}", conversation_uuid, message.timestamp.timestamp_micros(), message.uuid);

        let config = config::standard();
        let encode = bincode::encode_to_vec(message, config).unwrap();
        // let serialized = standard().ser
        tree.insert(&key.as_bytes(), encode).expect("Failed to save message");
        Ok(())
    }

    pub fn get_conversation(&self, db_uuid: &Uuid) -> Result<Vec<Message>, String> {
        let tree = self.db.open_tree("messages").expect("Unable to open messages");
        let prefix = format!("{}:", db_uuid);
        let config = config::standard();
        let mut messages = Vec::new();
        for entry in tree.scan_prefix(prefix.as_bytes()) {
            let (_, bytes) = entry.ok().expect("Unable to read messages");
            let (message, _): (Message, usize) = bincode::decode_from_slice(&bytes[..], config).unwrap();
            messages.push(message);
        }
        Ok(messages)
    }

    pub fn remove_conversation(&self, conversation_id: &Uuid) -> Result<(), String> {
        let messages_tree = self.db.open_tree("messages").expect("Unable to open messages");

        let prefix = format!("{}:", conversation_id);
        let message_prefix = prefix.as_bytes();
        debug!("Removing conversation with prefix {}", prefix);
        let keys_to_remove: Vec<_> = messages_tree
            .scan_prefix(message_prefix)
            .keys()
            .collect::<Result<Vec<_>, sled::Error>>().expect("Unable to retrieve keys");

        for key in keys_to_remove {
            messages_tree.remove(&key).expect("Unable to remove message");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::db_element::chat_storage::ChatStorage;
    use tempfile::tempdir;
    use uuid::Uuid;
    use crate::db_element::chat::{Message, Sender};

    fn setup_chat_storage() -> ChatStorage {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test_db");
        ChatStorage::new(db_path).expect("Failed to initialize ChatStorage")
    }

    #[test]
    fn test_add_and_get_message() {
        let chat_storage = setup_chat_storage();
        let conversation_id = Uuid::new_v4();
        let message = Message {
            uuid: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            content: "Hello, world!".to_string(),
            sender: Sender::System,
            is_sql: false,
        };

        chat_storage.add_message(&conversation_id, &message).expect("Failed to add message");
        let messages = chat_storage.get_conversation(&conversation_id).expect("Failed to get messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, world!");
    }

    #[test]
    fn test_remove_conversation() {
        let chat_storage = setup_chat_storage();
        let conversation_id = Uuid::new_v4();
        let message = Message {
            uuid: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            content: "This will be deleted".to_string(),
            sender: Sender::System,
            is_sql: false,
        };

        chat_storage.add_message(&conversation_id, &message).expect("Failed to add message");
        chat_storage.remove_conversation(&conversation_id).expect("Failed to remove conversation");
        let messages = chat_storage.get_conversation(&conversation_id).expect("Failed to get messages");

        assert!(messages.is_empty());
    }

}