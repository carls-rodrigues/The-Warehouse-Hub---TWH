use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainError {
    ValidationError(String),
    BusinessLogicError(String),
    NotFound(String),
    Conflict(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DomainError::BusinessLogicError(msg) => write!(f, "Business logic error: {}", msg),
            DomainError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DomainError::Conflict(msg) => write!(f, "Conflict: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}
