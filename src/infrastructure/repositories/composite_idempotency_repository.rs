use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entities::idempotency::IdempotencyKey;
use crate::domain::services::idempotency_repository::IdempotencyRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct CompositeIdempotencyRepository<R: IdempotencyRepository, P: IdempotencyRepository> {
    primary: Arc<R>,   // Redis repository (primary)
    fallback: Arc<P>,  // PostgreSQL repository (fallback)
}

impl<R: IdempotencyRepository, P: IdempotencyRepository> CompositeIdempotencyRepository<R, P> {
    pub fn new(primary: Arc<R>, fallback: Arc<P>) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl<R: IdempotencyRepository, P: IdempotencyRepository> IdempotencyRepository for CompositeIdempotencyRepository<R, P> {
    async fn store_key(&self, key: &IdempotencyKey) -> Result<(), DomainError> {
        // Try primary (Redis) first
        match self.primary.store_key(key).await {
            Ok(()) => Ok(()),
            Err(DomainError::ValidationError(_)) => {
                // Redis failed, try fallback (PostgreSQL)
                self.fallback.store_key(key).await
            }
            Err(e) => Err(e), // Other errors (validation, conflict) are returned as-is
        }
    }

    async fn get_key(&self, idempotency_key: &str) -> Result<Option<IdempotencyKey>, DomainError> {
        // Try primary (Redis) first
        match self.primary.get_key(idempotency_key).await {
            Ok(Some(key)) => Ok(Some(key)),
            Ok(None) => {
                // Not found in Redis, try fallback (PostgreSQL)
                self.fallback.get_key(idempotency_key).await
            }
            Err(DomainError::ValidationError(_)) => {
                // Redis failed, try fallback (PostgreSQL)
                self.fallback.get_key(idempotency_key).await
            }
            Err(e) => Err(e), // Other errors are returned as-is
        }
    }

    async fn complete_key(
        &self,
        idempotency_key: &str,
        status: i32,
        body: Option<String>,
    ) -> Result<(), DomainError> {
        // Try primary (Redis) first
        match self.primary.complete_key(idempotency_key, status.clone(), body.clone()).await {
            Ok(()) => Ok(()),
            Err(DomainError::ValidationError(_)) => {
                // Redis failed, try fallback (PostgreSQL)
                self.fallback.complete_key(idempotency_key, status, body).await
            }
            Err(e) => Err(e), // Other errors are returned as-is
        }
    }

    async fn delete_expired_keys(&self) -> Result<i64, DomainError> {
        // Clean up both stores
        let primary_result = self.primary.delete_expired_keys().await;
        let fallback_result = self.fallback.delete_expired_keys().await;

        // Return the sum of deleted keys, or the first error encountered
        match (primary_result, fallback_result) {
            (Ok(primary_count), Ok(fallback_count)) => Ok(primary_count + fallback_count),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }

    async fn key_exists(&self, idempotency_key: &str) -> Result<bool, DomainError> {
        // Try primary (Redis) first
        match self.primary.key_exists(idempotency_key).await {
            Ok(true) => Ok(true),
            Ok(false) => {
                // Not found in Redis, try fallback (PostgreSQL)
                self.fallback.key_exists(idempotency_key).await
            }
            Err(DomainError::ValidationError(_)) => {
                // Redis failed, try fallback (PostgreSQL)
                self.fallback.key_exists(idempotency_key).await
            }
            Err(e) => Err(e), // Other errors are returned as-is
        }
    }
}