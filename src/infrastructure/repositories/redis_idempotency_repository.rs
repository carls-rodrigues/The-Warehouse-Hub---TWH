use std::sync::Arc;

use async_trait::async_trait;
use redis::{Client as RedisClient, Commands};
use serde_json;

use crate::domain::entities::idempotency::{IdempotencyKey, IdempotencyKeyRequest};
use crate::domain::services::idempotency_repository::IdempotencyRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct RedisIdempotencyRepository {
    client: Arc<RedisClient>,
}

impl RedisIdempotencyRepository {
    pub fn new(redis_url: &str) -> Result<Self, DomainError> {
        let client = RedisClient::open(redis_url)
            .map_err(|e| DomainError::ValidationError(format!("Redis connection error: {}", e)))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    async fn get_connection(&self) -> Result<redis::Connection, DomainError> {
        let client = Arc::clone(&self.client);
        tokio::task::spawn_blocking(move || {
            client
                .get_connection()
                .map_err(|e| DomainError::ValidationError(format!("Redis connection error: {}", e)))
        })
        .await
        .map_err(|e| DomainError::ValidationError(format!("Task join error: {}", e)))?
    }
}

#[async_trait]
impl IdempotencyRepository for RedisIdempotencyRepository {
    async fn store_key(&self, key: &IdempotencyKey) -> Result<(), DomainError> {
        let key_data = serde_json::to_string(key)
            .map_err(|e| DomainError::ValidationError(format!("Failed to serialize key: {}", e)))?;

        let key_clone = key.idempotency_key.clone();
        let key_data_clone = key_data.clone();
        let ttl_seconds = (key.expires_at.timestamp() - chrono::Utc::now().timestamp()).max(1);

        let client = Arc::clone(&self.client);
        tokio::task::spawn_blocking(move || {
            let mut conn = client.get_connection().map_err(|e| {
                DomainError::ValidationError(format!("Redis connection error: {}", e))
            })?;

            let result: Result<(), redis::RedisError> = redis::cmd("SET")
                .arg(&key_clone)
                .arg(&key_data_clone)
                .arg("NX")
                .arg("EX")
                .arg(ttl_seconds)
                .query(&mut conn);

            match result {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == redis::ErrorKind::TypeError => Err(DomainError::Conflict(
                    "Idempotency key already exists".to_string(),
                )),
                Err(e) => Err(DomainError::ValidationError(format!("Redis error: {}", e))),
            }
        })
        .await
        .map_err(|e| DomainError::ValidationError(format!("Task join error: {}", e)))?
    }

    async fn get_key(&self, idempotency_key: &str) -> Result<Option<IdempotencyKey>, DomainError> {
        let key = idempotency_key.to_string();
        let client = Arc::clone(&self.client);

        let result = tokio::task::spawn_blocking(move || {
            let mut conn = client.get_connection().map_err(|e| {
                DomainError::ValidationError(format!("Redis connection error: {}", e))
            })?;

            let key_data: Option<String> = conn
                .get(&key)
                .map_err(|e| DomainError::ValidationError(format!("Redis error: {}", e)))?;

            match key_data {
                Some(data) => {
                    let key: IdempotencyKey = serde_json::from_str(&data).map_err(|e| {
                        DomainError::ValidationError(format!("Failed to deserialize key: {}", e))
                    })?;

                    if key.is_expired() {
                        // Key is expired, remove it
                        let _: () = conn.del(key.idempotency_key).map_err(|e| {
                            DomainError::ValidationError(format!("Redis error: {}", e))
                        })?;
                        Ok(None)
                    } else {
                        Ok(Some(key))
                    }
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| DomainError::ValidationError(format!("Task join error: {}", e)))?;

        result
    }

    async fn complete_key(
        &self,
        idempotency_key: &str,
        status: i32,
        body: Option<String>,
    ) -> Result<(), DomainError> {
        let key = idempotency_key.to_string();
        let body_clone = body.clone();
        let client = Arc::clone(&self.client);

        tokio::task::spawn_blocking(move || {
            let mut conn = client.get_connection().map_err(|e| {
                DomainError::ValidationError(format!("Redis connection error: {}", e))
            })?;

            // Get the current key
            let key_data: Option<String> = conn
                .get(&key)
                .map_err(|e| DomainError::ValidationError(format!("Redis error: {}", e)))?;

            if let Some(data) = key_data {
                let mut key: IdempotencyKey = serde_json::from_str(&data).map_err(|e| {
                    DomainError::ValidationError(format!("Failed to deserialize key: {}", e))
                })?;

                // Complete the key
                key.complete(status, body_clone)?;

                // Store the updated key
                let updated_data = serde_json::to_string(&key).map_err(|e| {
                    DomainError::ValidationError(format!("Failed to serialize key: {}", e))
                })?;

                let _: () = conn
                    .set(key.idempotency_key, updated_data)
                    .map_err(|e| DomainError::ValidationError(format!("Redis error: {}", e)))?;

                Ok(())
            } else {
                Err(DomainError::NotFound(
                    "Idempotency key not found".to_string(),
                ))
            }
        })
        .await
        .map_err(|e| DomainError::ValidationError(format!("Task join error: {}", e)))?
    }

    async fn delete_expired_keys(&self) -> Result<i64, DomainError> {
        // Redis automatically expires keys, so this is a no-op for Redis implementation
        // In a real implementation, you might want to scan for expired keys and clean them up
        Ok(0)
    }

    async fn key_exists(&self, idempotency_key: &str) -> Result<bool, DomainError> {
        let key = idempotency_key.to_string();
        let client = Arc::clone(&self.client);

        let result = tokio::task::spawn_blocking(move || {
            let mut conn = client.get_connection().map_err(|e| {
                DomainError::ValidationError(format!("Redis connection error: {}", e))
            })?;

            let exists: bool = conn
                .exists(&key)
                .map_err(|e| DomainError::ValidationError(format!("Redis error: {}", e)))?;

            Ok(exists)
        })
        .await
        .map_err(|e| DomainError::ValidationError(format!("Task join error: {}", e)))?;

        result
    }
}
