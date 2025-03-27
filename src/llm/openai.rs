use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::llm::llm::ContentResponse;

pub enum Model {
    Gpt4Turbo,
    Gpt4,
    Gpt35Turbo,
    Gpt35Turbo16k,
    TextDavinci003,
    TextDavinci002,
    CodeDavinci002,
}

impl Model {
    fn name(&self) -> &'static str {
        match self {
            Model::Gpt4Turbo => "gpt-4-turbo",
            Model::Gpt4 => "gpt-4",
            Model::Gpt35Turbo => "gpt-3.5-turbo",
            Model::Gpt35Turbo16k => "gpt-3.5-turbo-16k",
            Model::TextDavinci003 => "text-davinci-003",
            Model::TextDavinci002 => "text-davinci-002",
            Model::CodeDavinci002 => "code-davinci-002",
        }
    }

    pub fn variants() -> Vec<Model> {
        use Model::*;
        vec![
            Gpt4Turbo,
            Gpt4,
            Gpt35Turbo,
            Gpt35Turbo16k,
            TextDavinci003,
            TextDavinci002,
            CodeDavinci002,
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

#[derive(Serialize, Deserialize, Debug)]
struct OpenaiRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
}
pub async fn llm_request(api_key: String, client: &Client, model: String, user_query: &str, schema_info: &str) -> Result<Value, String> {
    let openai_prompt = format!(
        "You are a helpful database assistant. Convert natural language queries to SQL.
            Do not include any explanations. Always return a JSON object in this format:

            {{
                \"type\": \"query\",
                \"message\": \"SELECT * FROM users;\"
            }}
            OR
            {{
                \"type\": \"clarification\",
                \"message\": \"I need more details about the table you want to query.\"
            }}
            Use the following database schema information:
            {}

            Only write SQL that selects data, do not modify the database (no INSERT, UPDATE, DELETE, etc.).
            Return only the SQL query without backticks or any additional text.",
        schema_info
    );

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: openai_prompt,
        },
        Message {
            role: "user".to_string(),
            content: user_query.to_string(),
        },
    ];

    let request = OpenaiRequest {
        model,
        messages,
        max_tokens: 1000,
        temperature: 0.0, // Use low temperature for deterministic results
    };

    debug!("Sending Openai request: {:?}", request);
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    debug!("Openai response: {:?}", response);
    let response_json: Value = response.json().await.map_err(|e| {
        error!("Failed to parse response {}", e);
        e.to_string()
    })?;
    debug!("OpenAI response data: {}", response_json);
    Ok(response_json)

}

pub fn parse_content(response_json: Value) -> Result<ContentResponse, String> {
    if let Some(content) = response_json["choices"]
        .get(0)
        .and_then(|choice| choice["message"]["content"].as_str())
    {
        let parsed: ContentResponse = serde_json::from_str(content).map_err(|e| e.to_string())?;
        println!("{:?}", parsed);
        Ok(parsed)
    } else {
        Err("message does not contain content".to_string())
    }
}