use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::tenant::{Tenant, TenantType};
use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct CreateTenantUseCase<T: TenantRepository> {
    tenant_repository: Arc<T>,
}

impl<T: TenantRepository> CreateTenantUseCase<T> {
    pub fn new(tenant_repository: Arc<T>) -> Self {
        Self { tenant_repository }
    }

    pub async fn execute(
        &self,
        name: String,
        tenant_type: TenantType,
        created_by: Option<Uuid>,
    ) -> Result<Tenant, DomainError> {
        // Generate unique database schema name
        let database_schema = format!("tenant_{}", Uuid::new_v4().simple());

        let tenant = Tenant::new(name, tenant_type, database_schema, created_by)?;

        self.tenant_repository.create_tenant(&tenant).await?;

        Ok(tenant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::tenant::TenantType;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::domain::services::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_create_tenant_success() {
        let mut mock_repo = MockTenantRepository::new();
        mock_repo.expect_create_tenant().returning(|_| Ok(()));

        let use_case = CreateTenantUseCase::new(mock_repo);
        let result = use_case
            .execute("Test Tenant".to_string(), TenantType::Sandbox, None)
            .await;

        assert!(result.is_ok());
        let tenant = result.unwrap();
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.tenant_type, TenantType::Sandbox);
    }
}
