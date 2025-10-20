use crate::shared::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyKey {
    pub id: Uuid,
    pub idempotency_key: String,
    pub request_path: String,
    pub request_method: String,
    pub request_body_hash: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyKeyRequest {
    pub idempotency_key: String,
    pub request_path: String,
    pub request_method: String,
    pub request_body_hash: String,
    pub ttl_seconds: Option<i64>, // Time to live in seconds, defaults to 24 hours
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyKeyResponse {
    pub id: Uuid,
    pub idempotency_key: String,
    pub request_path: String,
    pub request_method: String,
    pub request_body_hash: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IdempotencyKey {
    pub fn new(request: IdempotencyKeyRequest) -> Result<Self, DomainError> {
        let ttl_seconds = request.ttl_seconds.unwrap_or(86400); // 24 hours default
        let expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds);
        let now = Utc::now();

        if request.idempotency_key.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Idempotency key cannot be empty".to_string(),
            ));
        }

        if request.idempotency_key.len() > 255 {
            return Err(DomainError::ValidationError(
                "Idempotency key cannot exceed 255 characters".to_string(),
            ));
        }

        if request.request_path.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Request path cannot be empty".to_string(),
            ));
        }

        if request.request_method.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Request method cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            idempotency_key: request.idempotency_key,
            request_path: request.request_path,
            request_method: request.request_method,
            request_body_hash: request.request_body_hash,
            response_status: None,
            response_body: None,
            expires_at,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn complete(&mut self, status: i32, body: Option<String>) -> Result<(), DomainError> {
        if self.response_status.is_some() {
            return Err(DomainError::ValidationError(
                "Idempotency key has already been completed".to_string(),
            ));
        }

        self.response_status = Some(status);
        self.response_body = body;
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl From<IdempotencyKey> for IdempotencyKeyResponse {
    fn from(key: IdempotencyKey) -> Self {
        Self {
            id: key.id,
            idempotency_key: key.idempotency_key,
            request_path: key.request_path,
            request_method: key.request_method,
            request_body_hash: key.request_body_hash,
            response_status: key.response_status,
            response_body: key.response_body,
            expires_at: key.expires_at,
            created_at: key.created_at,
            updated_at: key.updated_at,
        }
    }
}
