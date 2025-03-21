use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Sender {
    System,
    User
}
pub struct Message {
    pub sender: Sender,
    pub content: String,
    pub is_sql: bool,
    pub sent_time: String,
}

pub struct Chat {
    pub messages: Vec<Message>,
}