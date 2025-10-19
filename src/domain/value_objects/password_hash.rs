use crate::shared::error::DomainError;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Creates a new password hash from a plain text password
    pub fn new(password: &str) -> Result<Self, DomainError> {
        if password.len() < 8 {
            return Err(DomainError::ValidationError(
                "Password must be at least 8 characters long".to_string(),
            ));
        }

        if password.len() > 128 {
            return Err(DomainError::ValidationError(
                "Password is too long".to_string(),
            ));
        }

        // Hash the password with bcrypt
        match hash(password, DEFAULT_COST) {
            Ok(hashed) => Ok(Self(hashed)),
            Err(_) => Err(DomainError::ValidationError(
                "Failed to hash password".to_string(),
            )),
        }
    }

    /// Creates a PasswordHash from an existing hash (for database loading)
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Verifies a plain text password against this hash
    pub fn verify(&self, password: &str) -> Result<bool, DomainError> {
        match verify(password, &self.0) {
            Ok(result) => Ok(result),
            Err(_) => Err(DomainError::ValidationError(
                "Failed to verify password".to_string(),
            )),
        }
    }

    /// Returns the hash value as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the hash value as a String
    pub fn into_string(self) -> String {
        self.0
    }
}
