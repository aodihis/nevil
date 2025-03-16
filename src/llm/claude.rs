use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<crate::llm::llm::Message>,
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