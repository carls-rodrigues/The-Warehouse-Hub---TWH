use crate::domain::value_objects::{email::Email, password_hash::PasswordHash};
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: Email,
    pub password_hash: PasswordHash,
    pub first_name: String,
    pub last_name: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(
        email: Email,
        password_hash: PasswordHash,
        first_name: String,
        last_name: String,
    ) -> Result<Self, DomainError> {
        if first_name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "First name cannot be empty".to_string(),
            ));
        }

        if last_name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Last name cannot be empty".to_string(),
            ));
        }

        let now = chrono::Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            first_name,
            last_name,
            active: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, DomainError> {
        self.password_hash.verify(password)
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = chrono::Utc::now();
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = chrono::Utc::now();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}
