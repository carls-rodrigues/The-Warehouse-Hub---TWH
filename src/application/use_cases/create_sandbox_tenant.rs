use std::sync::Arc;
use uuid::Uuid;

use crate::application::use_cases::{
    create_item::{CreateItemRequest, CreateItemUseCase},
    create_location::{CreateLocationRequest, CreateLocationUseCase},
};
use crate::domain::entities::tenant::{Tenant, TenantType};
use crate::domain::services::item_repository::ItemRepository;
use crate::domain::services::location_repository::LocationRepository;
use crate::domain::services::tenant_repository::TenantRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct CreateSandboxTenantUseCase<T, I, L>
where
    T: TenantRepository,
    I: ItemRepository,
    L: LocationRepository,
{
    tenant_repository: Arc<T>,
    create_item_use_case: CreateItemUseCase<I>,
    create_location_use_case: CreateLocationUseCase<L>,
}

impl<T, I, L> CreateSandboxTenantUseCase<T, I, L>
where
    T: TenantRepository,
    I: ItemRepository,
    L: LocationRepository,
{
    pub fn new(
        tenant_repository: Arc<T>,
        create_item_use_case: CreateItemUseCase<I>,
        create_location_use_case: CreateLocationUseCase<L>,
    ) -> Self {
        Self {
            tenant_repository,
            create_item_use_case,
            create_location_use_case,
        }
    }

    pub async fn execute(&self, created_by: Option<Uuid>) -> Result<Tenant, DomainError> {
        // Create the sandbox tenant
        let tenant = Tenant::new_sandbox(created_by);
        self.tenant_repository.create_tenant(&tenant).await?;

        // Populate with sample data
        self.populate_sample_data(&tenant).await?;

        // Mark tenant as active after sample data is loaded
        self.tenant_repository
            .update_tenant_status(tenant.id, "ACTIVE")
            .await?;

        Ok(tenant)
    }

    async fn populate_sample_data(&self, _tenant: &Tenant) -> Result<(), DomainError> {
        use crate::domain::entities::location::LocationAddress;

        // Create sample locations
        let warehouse_request = CreateLocationRequest {
            name: "Main Warehouse".to_string(),
            code: Some("WH-001".to_string()),
            address: Some(LocationAddress {
                line1: Some("123 Industrial Ave".to_string()),
                line2: None,
                city: Some("Industrial City".to_string()),
                region: Some("IC".to_string()),
                postal_code: Some("12345".to_string()),
                country: Some("USA".to_string()),
            }),
            r#type: Some("warehouse".to_string()),
        };
        let _warehouse_location = self
            .create_location_use_case
            .execute(warehouse_request)
            .await?;

        let retail_request = CreateLocationRequest {
            name: "Retail Store".to_string(),
            code: Some("RT-001".to_string()),
            address: Some(LocationAddress {
                line1: Some("456 Commerce St".to_string()),
                line2: None,
                city: Some("Commerce City".to_string()),
                region: Some("CC".to_string()),
                postal_code: Some("67890".to_string()),
                country: Some("USA".to_string()),
            }),
            r#type: Some("store".to_string()),
        };
        let _retail_store = self
            .create_location_use_case
            .execute(retail_request)
            .await?;

        // Create sample items
        let laptop_request = CreateItemRequest {
            sku: "LPT-001".to_string(),
            name: "Gaming Laptop".to_string(),
            description: Some("High-performance gaming laptop".to_string()),
            category: Some("Electronics".to_string()),
            unit: "each".to_string(),
            barcode: Some("123456789012".to_string()),
            cost_price: 899.99,
            sale_price: Some(1299.99),
            reorder_point: Some(5),
            reorder_qty: Some(10),
            weight: Some(2.5),
            dimensions: None,
            metadata: None,
        };
        let _laptop = self.create_item_use_case.execute(laptop_request, _tenant.id).await?;

        let mouse_request = CreateItemRequest {
            sku: "MSE-001".to_string(),
            name: "Wireless Mouse".to_string(),
            description: Some("Ergonomic wireless mouse".to_string()),
            category: Some("Electronics".to_string()),
            unit: "each".to_string(),
            barcode: Some("123456789013".to_string()),
            cost_price: 29.99,
            sale_price: Some(49.99),
            reorder_point: Some(20),
            reorder_qty: Some(50),
            weight: Some(0.1),
            dimensions: None,
            metadata: None,
        };
        let _mouse = self.create_item_use_case.execute(mouse_request, _tenant.id).await?;

        let keyboard_request = CreateItemRequest {
            sku: "KBD-001".to_string(),
            name: "Mechanical Keyboard".to_string(),
            description: Some("RGB mechanical gaming keyboard".to_string()),
            category: Some("Electronics".to_string()),
            unit: "each".to_string(),
            barcode: Some("123456789014".to_string()),
            cost_price: 79.99,
            sale_price: Some(129.99),
            reorder_point: Some(10),
            reorder_qty: Some(25),
            weight: Some(0.8),
            dimensions: None,
            metadata: None,
        };
        let _keyboard = self.create_item_use_case.execute(keyboard_request, _tenant.id).await?;

        let tshirt_request = CreateItemRequest {
            sku: "TSH-001".to_string(),
            name: "Cotton T-Shirt".to_string(),
            description: Some("Comfortable cotton t-shirt".to_string()),
            category: Some("Apparel".to_string()),
            unit: "each".to_string(),
            barcode: Some("123456789015".to_string()),
            cost_price: 9.99,
            sale_price: Some(19.99),
            reorder_point: Some(50),
            reorder_qty: Some(100),
            weight: Some(0.2),
            dimensions: None,
            metadata: None,
        };
        let _tshirt = self.create_item_use_case.execute(tshirt_request, _tenant.id).await?;

        // TODO: Create initial stock levels for these items
        // This would require a create_stock_adjustment use case

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::services::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_create_sandbox_tenant_success() {
        let mut mock_repo = MockTenantRepository::new();
        mock_repo.expect_create_tenant().returning(|_| Ok(()));

        // TODO: Add mock use cases for items and locations
        // let create_item_use_case = CreateItemUseCase::new(mock_repo.clone());
        // let create_location_use_case = CreateLocationUseCase::new(mock_repo.clone());

        // let use_case = CreateSandboxTenantUseCase::new(
        //     mock_repo,
        //     create_item_use_case,
        //     create_location_use_case,
        // );

        // For now, just test tenant creation
        // let result = use_case.execute(None).await;
        // assert!(result.is_ok());
    }
}
