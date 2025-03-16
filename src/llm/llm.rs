use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::llm::claude;
use crate::config::LLMConfig;
use crate::security::SecureStorage;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Provider {
    OpenAI,
    Claude,
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Claude => "Claude",
        }
    }
}

#[derive(Clone)]
pub struct LLMClient {
    client: Client,
    config: LLMConfig,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    role: String,
    content: String,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn generate_sql(&self, user_query: &str, schema_info: &str) -> Result<String, String> {
        // Retrieve the API key securely
        let api_key = match SecureStorage::get_api_key() {
            Ok(key) => key,
            Err(_) => return Err("API key not found".to_string()),
        };

        // Build system prompt that instructs the LLM to convert natural language to SQL
        let system_prompt = format!(
            "You are a helpful database assistant. Convert natural language queries to SQL.
            Do not include any explanations, just return the SQL query.
            Use the following database schema information:
            {}

            Only write SQL that selects data, do not modify the database (no INSERT, UPDATE, DELETE, etc.).
            Return only the SQL query without backticks or any additional text.",
            schema_info
        );

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_query.to_string(),
            },
        ];

        let provider = self.config.provider.clone().expect("LLM configuration missing");

        match provider {
            Provider::OpenAI => {},
            Provider::Claude => {}
        }

        let request = claude::ClaudeRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: 1000,
            temperature: 0.0, // Use low temperature for deterministic results
        };

        // Send request to Claude API
        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        // Parse the response
        let response_json: Value = response.json().await.map_err(|e| e.to_string())?;

        // Extract the SQL query
        if let Some(content) = response_json["content"][0]["text"].as_str() {
            Ok(content.trim().to_string())
        } else {
            Err("Failed to parse LLM response".to_string())
        }
    }
}