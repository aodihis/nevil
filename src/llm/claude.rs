use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub enum Model {
    Claude37,
}

impl Model {
    pub fn name(&self) ->  &'static str {
        match self {
            Model::Claude37 => "claude-3-7-sonnet-20250219",
        }
    }
    pub fn variants() -> Vec<Model> {
        use Model::*;
        vec![
            Claude37
        ]
    }

    pub fn variants_name() -> Vec<&'static str> {
        Self::variants().iter().map(|model| model.name()).collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}

pub async fn llm_request(api_key: String, client: &Client, model: String, user_query: &str, schema_info: &str) -> Result<Value, String> {
    let claude_prompt = format!(
        "You are a helpful database assistant. Convert natural language queries to SQL.
        Do not include any explanations, just return the SQL query.
        Use the following database schema information:
        {}

        Only write SQL that selects data, do not modify the database (no INSERT, UPDATE, DELETE, etc.).
        Return only the SQL query without backticks or any additional text.

        Here is the user's request: {}",
            schema_info,
            user_query
    );

    let messages = vec![
        Message {
            role: "user".to_string(),
            content: claude_prompt,
        },
    ];

    let request = ClaudeRequest {
        model,
        messages,
        max_tokens: 1000,
        temperature: 0.0, // Use low temperature for deterministic results
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response_json: Value = response.json().await.map_err(|e| e.to_string())?;

    Ok(response_json)

}