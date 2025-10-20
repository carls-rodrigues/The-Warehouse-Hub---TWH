use std::sync::Arc;

use crate::domain::entities::idempotency::{
    IdempotencyKey, IdempotencyKeyRequest, IdempotencyKeyResponse,
};
use crate::domain::services::idempotency_repository::IdempotencyRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct IdempotencyUseCase<R: IdempotencyRepository> {
    repository: Arc<R>,
}

impl<R: IdempotencyRepository> IdempotencyUseCase<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn create_key(
        &self,
        request: IdempotencyKeyRequest,
    ) -> Result<IdempotencyKeyResponse, DomainError> {
        let key = IdempotencyKey::new(request)?;
        self.repository.store_key(&key).await?;
        Ok(key.into())
    }

    pub async fn get_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<IdempotencyKeyResponse>, DomainError> {
        match self.repository.get_key(idempotency_key).await? {
            Some(key) => Ok(Some(key.into())),
            None => Ok(None),
        }
    }

    pub async fn complete_key(
        &self,
        idempotency_key: &str,
        status: i32,
        body: Option<String>,
    ) -> Result<(), DomainError> {
        self.repository
            .complete_key(idempotency_key, status, body)
            .await
    }

    pub async fn cleanup_expired_keys(&self) -> Result<i64, DomainError> {
        self.repository.delete_expired_keys().await
    }

    pub async fn key_exists(&self, idempotency_key: &str) -> Result<bool, DomainError> {
        self.repository.key_exists(idempotency_key).await
    }
}
