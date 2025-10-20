use crate::domain::entities::idempotency::{IdempotencyKey, IdempotencyKeyRequest};
use crate::shared::error::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait IdempotencyRepository: Send + Sync {
    /// Store a new idempotency key (returns error if key already exists)
    async fn store_key(&self, key: &IdempotencyKey) -> Result<(), DomainError>;

    /// Get an idempotency key by its key value
    async fn get_key(&self, idempotency_key: &str) -> Result<Option<IdempotencyKey>, DomainError>;

    /// Complete an idempotency key with response data
    async fn complete_key(
        &self,
        idempotency_key: &str,
        status: i32,
        body: Option<String>,
    ) -> Result<(), DomainError>;

    /// Delete expired idempotency keys
    async fn delete_expired_keys(&self) -> Result<i64, DomainError>;

    /// Check if a key exists and is not expired
    async fn key_exists(&self, idempotency_key: &str) -> Result<bool, DomainError>;
}
