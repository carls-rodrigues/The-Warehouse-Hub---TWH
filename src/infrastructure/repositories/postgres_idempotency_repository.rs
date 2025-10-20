use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::entities::idempotency::{IdempotencyKey, IdempotencyKeyRequest};
use crate::domain::services::idempotency_repository::IdempotencyRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct PostgresIdempotencyRepository {
    pool: Arc<PgPool>,
}

impl PostgresIdempotencyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IdempotencyRepository for PostgresIdempotencyRepository {
    async fn store_key(&self, key: &IdempotencyKey) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            INSERT INTO idempotency_keys (
                id, idempotency_key, request_path, request_method,
                request_body_hash, response_status, response_body,
                expires_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (idempotency_key) DO NOTHING
            "#,
        )
        .bind(key.id)
        .bind(&key.idempotency_key)
        .bind(&key.request_path)
        .bind(&key.request_method)
        .bind(&key.request_body_hash)
        .bind(key.response_status)
        .bind(&key.response_body)
        .bind(key.expires_at)
        .bind(key.created_at)
        .bind(key.updated_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Failed to store idempotency key: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::Conflict("Idempotency key already exists".to_string()));
        }

        Ok(())
    }

    async fn get_key(&self, idempotency_key: &str) -> Result<Option<IdempotencyKey>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, idempotency_key, request_path, request_method,
                   request_body_hash, response_status, response_body,
                   expires_at, created_at, updated_at
            FROM idempotency_keys
            WHERE idempotency_key = $1 AND expires_at > NOW()
            "#,
        )
        .bind(idempotency_key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Failed to get idempotency key: {}", e)))?;

        match row {
            Some(row) => {
                let key = IdempotencyKey {
                    id: row.try_get("id").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    idempotency_key: row.try_get("idempotency_key").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    request_path: row.try_get("request_path").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    request_method: row.try_get("request_method").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    request_body_hash: row.try_get("request_body_hash").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    response_status: row.try_get("response_status").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    response_body: row.try_get("response_body").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    expires_at: row.try_get("expires_at").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    created_at: row.try_get("created_at").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    updated_at: row.try_get("updated_at").map_err(|e| DomainError::ValidationError(e.to_string()))?,
                };

                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    async fn complete_key(
        &self,
        idempotency_key: &str,
        status: i32,
        body: Option<String>,
    ) -> Result<(), DomainError> {
        let result = sqlx::query(
            r#"
            UPDATE idempotency_keys
            SET response_status = $1, response_body = $2, updated_at = NOW()
            WHERE idempotency_key = $3 AND response_status IS NULL
            "#,
        )
        .bind(status)
        .bind(&body)
        .bind(idempotency_key)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Failed to complete idempotency key: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound("Idempotency key not found or already completed".to_string()));
        }

        Ok(())
    }

    async fn delete_expired_keys(&self) -> Result<i64, DomainError> {
        let result = sqlx::query("DELETE FROM idempotency_keys WHERE expires_at <= NOW()")
            .execute(&*self.pool)
            .await
            .map_err(|e| DomainError::ValidationError(format!("Failed to delete expired keys: {}", e)))?;

        Ok(result.rows_affected() as i64)
    }

    async fn key_exists(&self, idempotency_key: &str) -> Result<bool, DomainError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM idempotency_keys WHERE idempotency_key = $1 AND expires_at > NOW()",
        )
        .bind(idempotency_key)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Failed to check key existence: {}", e)))?;

        Ok(count.0 > 0)
    }
}