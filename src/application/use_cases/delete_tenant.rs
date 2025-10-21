use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct DeleteTenantUseCase<T: TenantRepository> {
    tenant_repository: Arc<T>,
}

impl<T: TenantRepository> DeleteTenantUseCase<T> {
    pub fn new(tenant_repository: Arc<T>) -> Self {
        Self { tenant_repository }
    }

    pub async fn execute(&self, tenant_id: Uuid) -> Result<(), DomainError> {
        // First check if tenant exists
        let tenant = self.tenant_repository.get_tenant(tenant_id).await?;
        if tenant.is_none() {
            return Err(DomainError::NotFound(format!(
                "Tenant {} not found",
                tenant_id
            )));
        }

        // Mark tenant for deletion (soft delete)
        self.tenant_repository.delete_tenant(tenant_id).await?;

        // TODO: In a real implementation, this would trigger:
        // 1. Background job to clean up tenant data
        // 2. Queue tenant schema deletion
        // 3. Notify dependent services

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::tenant::{Tenant, TenantStatus, TenantType};
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::services::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_delete_tenant_success() {
        let tenant_id = Uuid::new_v4();
        let tenant = Tenant {
            id: tenant_id,
            name: "Test Tenant".to_string(),
            tenant_type: TenantType::Sandbox,
            status: TenantStatus::Active,
            database_schema: "tenant_123".to_string(),
            created_by: Uuid::new_v4(),
            expires_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_get_tenant()
            .returning(move |_| Ok(Some(tenant.clone())));
        mock_repo.expect_delete_tenant().returning(|_| Ok(()));

        let use_case = DeleteTenantUseCase::new(mock_repo);
        let result = use_case.execute(tenant_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_tenant_not_found() {
        let tenant_id = Uuid::new_v4();

        let mut mock_repo = MockTenantRepository::new();
        mock_repo.expect_get_tenant().returning(|_| Ok(None));

        let use_case = DeleteTenantUseCase::new(mock_repo);
        let result = use_case.execute(tenant_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::NotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected NotFound error"),
        }
    }
}
