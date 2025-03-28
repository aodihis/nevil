use keyring::Entry;
use std::fmt;

enum Service {
    DbAssistant,
    LlmAPI
}

impl Service {
    fn as_str(&self) -> &'static str {
        match *self {
            Service::DbAssistant => "nevil::db_element",
            Service::LlmAPI => "nevil::llm_api",
        }
    }
}
pub struct SecureStorage;

impl SecureStorage {
    // Store DB password securely
    pub fn store_db_password(uuid: &str, password: &str) -> Result<(), SecurityError> {
        let entry = Entry::new(Service::DbAssistant.as_str(), uuid)?;
        entry.set_password(password)?;
        Ok(())
    }

    // Retrieve DB password securely
    pub fn get_db_password(uuid: &str) -> Result<String, SecurityError> {
        let entry = Entry::new(Service::DbAssistant.as_str(), uuid)?;
        let password = entry.get_password()?;
        Ok(password)
    }

    pub fn remove_db_password(uuid: &str) -> Result<(), SecurityError> {
        let entry = Entry::new(Service::DbAssistant.as_str(), uuid)?;
        entry.delete_credential()?;
        Ok(())
    }

    // Store API key securely
    pub fn store_api_key(api_key: &str) -> Result<(), SecurityError> {
        let entry = Entry::new(Service::LlmAPI.as_str(), "api_key")?;
        entry.set_password(api_key)?;
        Ok(())
    }

    // Retrieve API key securely
    pub fn get_api_key() -> Result<String, SecurityError> {
        let entry = Entry::new(Service::LlmAPI.as_str(), "api_key")?;
        let api_key = entry.get_password()?;
        Ok(api_key)
    }
}

#[derive(Debug)]
pub enum SecurityError {
    KeyringError(keyring::Error),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::security::SecureStorage;

    #[test]
    fn db_password() {
        let pass = "password";
        let uuid = Uuid::new_v4();
        let res = SecureStorage::store_db_password(&uuid.to_string(), pass);

        assert!(res.is_ok());

        let res = SecureStorage::get_db_password(&uuid.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), pass);

        let res = SecureStorage::remove_db_password(&uuid.to_string());
        assert!(res.is_ok());
    }


}