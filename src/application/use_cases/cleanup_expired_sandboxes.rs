use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::tenant::Tenant;
use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct CleanupExpiredSandboxesUseCase<T: TenantRepository> {
    tenant_repository: Arc<T>,
}

impl<T: TenantRepository> CleanupExpiredSandboxesUseCase<T> {
    pub fn new(tenant_repository: Arc<T>) -> Self {
        Self { tenant_repository }
    }

    pub async fn execute(&self) -> Result<Vec<Uuid>, DomainError> {
        // Get all expired sandbox tenants
        let expired_tenants = self.tenant_repository.get_expired_sandboxes().await?;

        let mut cleaned_up_tenant_ids = Vec::new();

        for tenant in expired_tenants {
            // Mark tenant for deletion
            self.tenant_repository.delete_tenant(tenant.id).await?;
            cleaned_up_tenant_ids.push(tenant.id);

            // TODO: In a real implementation, this would also:
            // 1. Drop the tenant's database schema
            // 2. Clean up any tenant-specific resources
            // 3. Send cleanup notifications
        }

        Ok(cleaned_up_tenant_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::tenant::{Tenant, TenantStatus, TenantType};
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::services::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_cleanup_expired_sandboxes_success() {
        let tenant_id = Uuid::new_v4();
        let expired_tenant = Tenant {
            id: tenant_id,
            name: "Expired Sandbox".to_string(),
            tenant_type: TenantType::Sandbox,
            status: TenantStatus::Active,
            database_schema: "tenant_123".to_string(),
            created_by: Uuid::new_v4(),
            expires_at: Some(Utc::now() - Duration::days(1)), // Already expired
            created_at: Utc::now() - Duration::days(31),
            updated_at: Utc::now() - Duration::days(31),
        };

        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_get_expired_sandboxes()
            .returning(move || Ok(vec![expired_tenant.clone()]));
        mock_repo.expect_delete_tenant().returning(|_| Ok(()));

        let use_case = CleanupExpiredSandboxesUseCase::new(mock_repo);
        let result = use_case.execute().await;

        assert!(result.is_ok());
        let cleaned_ids = result.unwrap();
        assert_eq!(cleaned_ids.len(), 1);
        assert_eq!(cleaned_ids[0], tenant_id);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sandboxes_no_expired() {
        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_get_expired_sandboxes()
            .returning(|| Ok(vec![]));

        let use_case = CleanupExpiredSandboxesUseCase::new(mock_repo);
        let result = use_case.execute().await;

        assert!(result.is_ok());
        let cleaned_ids = result.unwrap();
        assert_eq!(cleaned_ids.len(), 0);
    }
}
