use keyring::Entry;
use std::fmt;

pub struct SecureStorage;

impl SecureStorage {
    // Store DB password securely
    pub fn store_db_password(connection_name: &str, password: &str) -> Result<(), SecurityError> {
        let entry = Entry::new("db_query_assistant", connection_name)?;
        entry.set_password(password)?;
        Ok(())
    }

    // Retrieve DB password securely
    pub fn get_db_password(connection_name: &str) -> Result<String, SecurityError> {
        let entry = Entry::new("db_query_assistant", connection_name)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    // Store API key securely
    pub fn store_api_key(api_key: &str) -> Result<(), SecurityError> {
        let entry = Entry::new("db_query_assistant_api", "llm_api_key")?;
        entry.set_password(api_key)?;
        Ok(())
    }

    // Retrieve API key securely
    pub fn get_api_key() -> Result<String, SecurityError> {
        let entry = Entry::new("db_query_assistant_api", "llm_api_key")?;
        let api_key = entry.get_password()?;
        Ok(api_key)
    }
}

#[derive(Debug)]
pub enum SecurityError {
    KeyringError(keyring::Error),
    EncryptionError(String),
}

impl From<keyring::Error> for SecurityError {
    fn from(err: keyring::Error) -> Self {
        SecurityError::KeyringError(err)
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::KeyringError(e) => write!(f, "Keyring error: {}", e),
            SecurityError::EncryptionError(e) => write!(f, "Encryption error: {}", e),
        }
    }
}