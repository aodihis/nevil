use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub llm_api: LLMConfig,
    pub connections: Vec<DbConnection>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub api_key: String, // Store encrypted or as a placeholder
    pub model: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DbConnection {
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
    pub connection_string_template: String,
    // Password will be stored securely separately
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum DbType {
    MySQL,
    PostgreSQL,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = get_config_path();
        if config_path.exists() {
            let config_str = fs::read_to_string(config_path).unwrap_or_default();
            toml::from_str(&config_str).unwrap_or_else(|_| Self::default())
        } else {
            let default = Self::default();
            default.save();
            default
        }
    }

    pub fn save(&self) {
        let config_path = get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let config_str = toml::to_string(self).unwrap_or_default();
        fs::write(config_path, config_str).ok();
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm_api: LLMConfig {
                provider: "Claude".to_string(),
                api_key: "".to_string(),
                model: "claude-3-7-sonnet-20250219".to_string(),
            },
            connections: Vec::new(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("./"));
    path.push("db_query_assistant");
    path.push("config.toml");
    path
}