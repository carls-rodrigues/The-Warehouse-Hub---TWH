use crate::domain::entities::item::{Item, ItemDimensions};
use crate::domain::services::item_repository::ItemRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateItemRequest {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: String,
    pub barcode: Option<String>,
    pub cost_price: f64,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<ItemDimensions>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateItemResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub cost_price: f64,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct CreateItemUseCase<R: ItemRepository> {
    item_repository: Arc<R>,
}

impl<R: ItemRepository> CreateItemUseCase<R> {
    pub fn new(item_repository: Arc<R>) -> Self {
        Self { item_repository }
    }

    pub async fn execute(
        &self,
        request: CreateItemRequest,
        tenant_id: Uuid,
    ) -> Result<CreateItemResponse, DomainError> {
        // Check if SKU already exists
        let sku_exists = self.item_repository.sku_exists(&request.sku, None).await?;
        if sku_exists {
            return Err(DomainError::ValidationError(format!(
                "Item with SKU '{}' already exists",
                request.sku
            )));
        }

        // Create the item with required fields
        let mut item = Item::new(
            tenant_id,
            request.sku,
            request.name,
            request.unit,
            request.cost_price,
        )?;

        // Update with optional fields
        let update_request = crate::domain::entities::item::UpdateItemRequest {
            sku: None,  // SKU is already set
            name: None, // Name is already set
            description: request.description,
            category: request.category,
            unit: None, // Unit is already set
            barcode: request.barcode,
            cost_price: None, // Cost price is already set
            sale_price: request.sale_price,
            reorder_point: request.reorder_point,
            reorder_qty: request.reorder_qty,
            weight: request.weight,
            dimensions: request.dimensions,
            metadata: request.metadata,
        };

        item.update(update_request)?;

        // Save to repository
        self.item_repository.save(&item).await?;

        // Return response
        Ok(CreateItemResponse {
            id: item.id,
            sku: item.sku,
            name: item.name,
            unit: item.unit,
            cost_price: item.cost_price,
            active: item.active,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}
