use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::tenant::Tenant;
use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct GetTenantUseCase<T: TenantRepository> {
    tenant_repository: Arc<T>,
}

impl<T: TenantRepository> GetTenantUseCase<T> {
    pub fn new(tenant_repository: Arc<T>) -> Self {
        Self { tenant_repository }
    }

    pub async fn execute(&self, tenant_id: Uuid) -> Result<Option<Tenant>, DomainError> {
        self.tenant_repository.get_tenant(tenant_id).await
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
    async fn test_get_tenant_success() {
        let tenant_id = Uuid::new_v4();
        let tenant = Tenant {
            id: tenant_id,
            name: "Test Tenant".to_string(),
            tenant_type: TenantType::Sandbox,
            status: TenantStatus::Active,
            database_schema: "tenant_123".to_string(),
            created_by: Uuid::new_v4(),
            expires_at: Some(Utc::now() + Duration::days(30)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_get_tenant()
            .returning(move |_| Ok(Some(tenant.clone())));

        let use_case = GetTenantUseCase::new(mock_repo);
        let result = use_case.execute(tenant_id).await;

        assert!(result.is_ok());
        let retrieved_tenant = result.unwrap().unwrap();
        assert_eq!(retrieved_tenant.id, tenant_id);
        assert_eq!(retrieved_tenant.name, "Test Tenant");
    }

    #[tokio::test]
    async fn test_get_tenant_not_found() {
        let tenant_id = Uuid::new_v4();

        let mut mock_repo = MockTenantRepository::new();
        mock_repo.expect_get_tenant().returning(|_| Ok(None));

        let use_case = GetTenantUseCase::new(mock_repo);
        let result = use_case.execute(tenant_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
