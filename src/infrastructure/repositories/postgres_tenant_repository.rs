use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::entities::tenant::{Tenant, TenantStatus, TenantType};
use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn create_tenant(&self, tenant: &Tenant) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO tenants (
                id, name, tenant_type, tier, status, database_schema,
                created_by, expires_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(tenant.id)
        .bind(&tenant.name)
        .bind(tenant.tenant_type.as_str())
        .bind(tenant.tier.as_str())
        .bind(tenant.status.as_str())
        .bind(&tenant.database_schema)
        .bind(tenant.created_by)
        .bind(tenant.expires_at)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, tenant_type, tier, status, database_schema,
                   created_by, expires_at, created_at, updated_at
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let tenant_type = TenantType::from_str(row.try_get("tenant_type")?)?;
            let tier = crate::domain::entities::tenant::TenantTier::from_str(row.try_get("tier")?)?;
            let status = TenantStatus::from_str(row.try_get("status")?)?;

            Ok(Some(Tenant {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                tenant_type,
                tier,
                status,
                database_schema: row.try_get("database_schema")?,
                created_by: row.try_get("created_by")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, tenant_type, tier, status, database_schema,
                   created_by, expires_at, created_at, updated_at
            FROM tenants ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let mut tenants = Vec::new();
        for row in rows {
            let tenant_type = TenantType::from_str(row.try_get("tenant_type")?)?;
            let tier = crate::domain::entities::tenant::TenantTier::from_str(row.try_get("tier")?)?;
            let status = TenantStatus::from_str(row.try_get("status")?)?;

            tenants.push(Tenant {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                tenant_type,
                tier,
                status,
                database_schema: row.try_get("database_schema")?,
                created_by: row.try_get("created_by")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(tenants)
    }

    async fn update_tenant_status(&self, tenant_id: Uuid, status: &str) -> Result<(), DomainError> {
        // Validate status
        TenantStatus::from_str(status)?;

        sqlx::query(
            r#"
            UPDATE tenants SET status = $1, updated_at = NOW() WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), DomainError> {
        // Mark as deleting rather than actually deleting
        sqlx::query(
            r#"
            UPDATE tenants SET status = 'DELETING', updated_at = NOW() WHERE id = $1
            "#,
        )
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_expired_sandboxes(&self) -> Result<Vec<Tenant>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, tenant_type, tier, status, database_schema,
                   created_by, expires_at, created_at, updated_at
            FROM tenants
            WHERE tenant_type = 'SANDBOX'
              AND expires_at IS NOT NULL
              AND expires_at < NOW()
              AND status != 'DELETING'
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let mut tenants = Vec::new();
        for row in rows {
            let tenant_type = TenantType::from_str(row.try_get("tenant_type")?)?;
            let tier = crate::domain::entities::tenant::TenantTier::from_str(row.try_get("tier")?)?;
            let status = TenantStatus::from_str(row.try_get("status")?)?;

            tenants.push(Tenant {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                tenant_type,
                tier,
                status,
                database_schema: row.try_get("database_schema")?,
                created_by: row.try_get("created_by")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(tenants)
    }

    async fn permanently_delete_tenant(&self, tenant_id: Uuid) -> Result<(), DomainError> {
        // This would delete all tenant data - use with extreme caution
        // In a real implementation, this would cascade delete all tenant-related data
        sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(tenant_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_tenant_tier(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<crate::domain::entities::tenant::TenantTier>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT tier FROM tenants WHERE id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let tier_str: String = row.try_get("tier")?;
            let tier = crate::domain::entities::tenant::TenantTier::from_str(&tier_str)?;
            Ok(Some(tier))
        } else {
            Ok(None)
        }
    }
}
