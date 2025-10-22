use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::tenant::Tenant;
use crate::shared::error::DomainError;

#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Create a new tenant
    async fn create_tenant(&self, tenant: &Tenant) -> Result<(), DomainError>;

    /// Get tenant by ID
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, DomainError>;

    /// List all tenants (admin only)
    async fn list_tenants(&self) -> Result<Vec<Tenant>, DomainError>;

    /// Update tenant status
    async fn update_tenant_status(&self, tenant_id: Uuid, status: &str) -> Result<(), DomainError>;

    /// Delete tenant (mark as deleting)
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), DomainError>;

    /// Get expired sandbox tenants for cleanup
    async fn get_expired_sandboxes(&self) -> Result<Vec<Tenant>, DomainError>;

    /// Permanently delete tenant data (for cleanup jobs)
    async fn permanently_delete_tenant(&self, tenant_id: Uuid) -> Result<(), DomainError>;

    /// Get tenant tier by ID (for rate limiting)
    async fn get_tenant_tier(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<crate::domain::entities::tenant::TenantTier>, DomainError>;
}

#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    pub TenantRepository {}

    #[async_trait]
    impl TenantRepository for TenantRepository {
        async fn create_tenant(&self, tenant: &Tenant) -> Result<(), DomainError>;
        async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, DomainError>;
        async fn list_tenants(&self) -> Result<Vec<Tenant>, DomainError>;
        async fn update_tenant_status(&self, tenant_id: Uuid, status: &str) -> Result<(), DomainError>;
        async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), DomainError>;
        async fn get_expired_sandboxes(&self) -> Result<Vec<Tenant>, DomainError>;
        async fn permanently_delete_tenant(&self, tenant_id: Uuid) -> Result<(), DomainError>;
        async fn get_tenant_tier(&self, tenant_id: Uuid) -> Result<Option<crate::domain::entities::tenant::TenantTier>, DomainError>;
    }
}
