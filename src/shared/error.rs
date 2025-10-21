use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainError {
    ValidationError(String),
    BusinessLogicError(String),
    NotFound(String),
    Conflict(String),
    InfrastructureError(String),
    DatabaseError(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            DomainError::BusinessLogicError(msg) => write!(f, "Business logic error: {msg}"),
            DomainError::NotFound(msg) => write!(f, "Not found: {msg}"),
            DomainError::Conflict(msg) => write!(f, "Conflict: {msg}"),
            DomainError::InfrastructureError(msg) => write!(f, "Infrastructure error: {msg}"),
            DomainError::DatabaseError(msg) => write!(f, "Database error: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        DomainError::DatabaseError(error.to_string())
    }
}
