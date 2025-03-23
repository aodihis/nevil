use crate::llm::llm::Provider;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub llm_api: LLMConfig,
    pub connections: Vec<DbConnection>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LLMConfig {
    pub provider: Option<Provider>,
    pub model: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DbConnection {
    pub uuid: Uuid,
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
}

impl DbConnection {
    pub fn connection_string_template(&self) -> String {
        match self.db_type {
            DbType::MySQL => {"mysql://{username}:{password}@{host}:{port}/{database}".to_string()}
            DbType::PostgreSQL => {"postgres://{username}:{password}@{host}:{port}/{database}".to_string()}
        }
    }
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
        println!("Saving config to: {}", config_path.display());
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
                provider: None,
                model: "".to_string(),
            },
            connections: Vec::new(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("./"));
    path.push("neVil");
    path.push("config.toml");
    path
}

pub fn get_chat_db_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("./"));
    path.push("neVil");
    path.push("db");
    path
}