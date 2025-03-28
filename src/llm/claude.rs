use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::llm::llm::ContentResponse;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    role: String,
    content: String,
}

pub async fn llm_request(api_key: String, client: &Client, model: String, user_query: &str, schema_info: &str) -> Result<Value, String> {
    let claude_prompt = format!(
        r#"
    You are a helpful database assistant. Convert natural language queries to SQL.
    Do not include any explanations. Always return a JSON array where each object follows this format:

    [
        {{
            "type": "query",
            "message": "SELECT * FROM users;"
        }},
        {{
            "type": "query",
            "message": "SELECT * FROM orders WHERE user_id = 1;"
        }}
    ]
    OR
    [
        {{
            "type": "clarification",
            "message": "I need more details about the table you want to query."
        }}
    ]

    Use the following database schema information:
    {}

    - Only write SQL that selects data (no INSERT, UPDATE, DELETE, etc.).
    - Return multiple queries as separate objects in the JSON array.
    - Ensure the response is valid JSON, without additional explanations or text.

    Here is the user's request: {}
    "#,
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


    debug!("Sending request to Claude: {:?}", request);
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
    debug!("Claude response: {:?}", response_json);
    Ok(response_json)

}

pub fn parse_content(response_json: Value) -> Result<Vec<ContentResponse>, String> {
    if let Some(content) = response_json["content"][0]["text"].as_str() {
        let parsed: Vec<ContentResponse> = serde_json::from_str(content).map_err(|e| e.to_string())?;
        Ok(parsed)
    } else {
        Err("message does not contain content".to_string())
    }
}