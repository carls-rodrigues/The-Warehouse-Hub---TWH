use crate::domain::entities::location::{Location, LocationAddress, LocationType};
use crate::domain::services::location_repository::LocationRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresLocationRepository {
    pool: Arc<PgPool>,
}

impl PostgresLocationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocationRepository for PostgresLocationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Location>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, name, code, address, type, active, created_at, updated_at
            FROM locations
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let address = row
                    .address
                    .map(|a| serde_json::from_value(a).unwrap_or_default());

                let r#type = row.r#type.map(|t| LocationType::from_str(&t)).transpose()?;

                Ok(Some(Location {
                    id: row.id,
                    name: row.name,
                    code: row.code,
                    address,
                    r#type,
                    active: row.active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Location>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, name, code, address, type, active, created_at, updated_at
            FROM locations
            WHERE code = $1
            "#,
            code
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let address = row
                    .address
                    .map(|a| serde_json::from_value(a).unwrap_or_default());

                let r#type = row.r#type.map(|t| LocationType::from_str(&t)).transpose()?;

                Ok(Some(Location {
                    id: row.id,
                    name: row.name,
                    code: row.code,
                    address,
                    r#type,
                    active: row.active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, location: &Location) -> Result<(), DomainError> {
        let address_json = location
            .address
            .as_ref()
            .map(|a| serde_json::to_value(a))
            .transpose()
            .map_err(|e| {
                DomainError::ValidationError(format!("Failed to serialize address: {}", e))
            })?;

        let type_str = location.r#type.as_ref().map(|t| t.as_str());

        sqlx::query!(
            r#"
            INSERT INTO locations (id, name, code, address, type, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            location.id,
            location.name,
            location.code,
            address_json,
            type_str,
            location.active,
            location.created_at,
            location.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update(&self, location: &Location) -> Result<(), DomainError> {
        let address_json = location
            .address
            .as_ref()
            .map(|a| serde_json::to_value(a))
            .transpose()
            .map_err(|e| {
                DomainError::ValidationError(format!("Failed to serialize address: {}", e))
            })?;

        let type_str = location.r#type.as_ref().map(|t| t.as_str());

        sqlx::query!(
            r#"
            UPDATE locations
            SET name = $2, code = $3, address = $4, type = $5, active = $6, updated_at = $7
            WHERE id = $1
            "#,
            location.id,
            location.name,
            location.code,
            address_json,
            type_str,
            location.active,
            location.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM locations
            WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Location>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, code, address, type, active, created_at, updated_at
            FROM locations
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut locations = Vec::new();
        for row in rows {
            let address = row
                .address
                .map(|a| serde_json::from_value(a).unwrap_or_default());

            let r#type = row.r#type.map(|t| LocationType::from_str(&t)).transpose()?;

            locations.push(Location {
                id: row.id,
                name: row.name,
                code: row.code,
                address,
                r#type,
                active: row.active,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(locations)
    }

    async fn count(&self) -> Result<i64, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM locations
            "#
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(result.count.unwrap_or(0))
    }

    async fn code_exists(
        &self,
        code: &str,
        exclude_location_id: Option<Uuid>,
    ) -> Result<bool, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM locations
            WHERE code = $1 AND ($2::uuid IS NULL OR id != $2)
            "#,
            code,
            exclude_location_id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(result.count.unwrap_or(0) > 0)
    }
}
