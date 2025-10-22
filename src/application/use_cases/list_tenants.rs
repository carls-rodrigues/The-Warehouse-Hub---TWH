use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::entities::tenant::Tenant;
use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct ListTenantsUseCase<T: TenantRepository> {
    tenant_repository: Arc<T>,
}

impl<T: TenantRepository> ListTenantsUseCase<T> {
    pub fn new(tenant_repository: Arc<T>) -> Self {
        Self { tenant_repository }
    }

    pub async fn execute(&self) -> Result<Vec<Tenant>, DomainError> {
        self.tenant_repository.list_tenants().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::tenant::{Tenant, TenantStatus, TenantTier, TenantType};
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::services::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_list_tenants_success() {
        let tenant = Tenant {
            id: Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            tenant_type: TenantType::Sandbox,
            tier: TenantTier::Free,
            status: TenantStatus::Active,
            database_schema: "tenant_123".to_string(),
            created_by: Some(Uuid::new_v4()),
            expires_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_list_tenants()
            .returning(move || Ok(vec![tenant.clone()]));

        let use_case = ListTenantsUseCase::new(mock_repo);
        let result = use_case.execute().await;

        assert!(result.is_ok());
        let tenants = result.unwrap();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0].name, "Test Tenant");
    }

    #[tokio::test]
    async fn test_list_tenants_empty() {
        let mut mock_repo = MockTenantRepository::new();
        mock_repo.expect_list_tenants().returning(|| Ok(vec![]));

        let use_case = ListTenantsUseCase::new(mock_repo);
        let result = use_case.execute().await;

        assert!(result.is_ok());
        let tenants = result.unwrap();
        assert_eq!(tenants.len(), 0);
    }
}
