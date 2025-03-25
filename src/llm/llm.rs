use crate::config::LLMConfig;
use crate::llm::{claude, openai};
use crate::security::SecureStorage;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

        let provider = self.config.provider.clone().expect("LLM configuration missing");

        let res = match provider {
            Provider::Claude => {
                claude::llm_request(api_key, &self.client, self.config.model.clone(), user_query, schema_info).await
            },
            Provider::OpenAI => {
                openai::llm_request(api_key, &self.client, self.config.model.clone(), user_query, schema_info).await
            }
        };

        let response_json = match res {
            Ok(json) => json,
            Err(_) => {
                return Err("Failed to generate LLM response".to_string());
            }
        };

        // Extract the SQL query
        if let Some(content) = response_json["content"][0]["text"].as_str() {
            Ok(content.trim().to_string())
        } else {
            Err("Failed to parse LLM response".to_string())
        }
    }
}