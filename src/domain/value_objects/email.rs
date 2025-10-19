use crate::shared::error::DomainError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Creates a new Email with validation
    pub fn new(value: String) -> Result<Self, DomainError> {
        if value.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Email cannot be empty".to_string(),
            ));
        }

        if value.len() > 254 {
            return Err(DomainError::ValidationError(
                "Email is too long".to_string(),
            ));
        }

        // RFC 5322 compliant email regex (simplified)
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(&value) {
            return Err(DomainError::ValidationError(
                "Invalid email format".to_string(),
            ));
        }

        Ok(Self(value.to_lowercase()))
    }

    /// Returns the email value as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the email value as a String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for Email {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
