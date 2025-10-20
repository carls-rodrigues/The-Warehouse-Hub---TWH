use crate::domain::services::item_repository::ItemRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemRequest {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetItemResponse {
    pub id: Uuid,
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
    pub dimensions: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetItemUseCase<R: ItemRepository> {
    item_repository: Arc<R>,
}

impl<R: ItemRepository> GetItemUseCase<R> {
    pub fn new(item_repository: Arc<R>) -> Self {
        Self { item_repository }
    }

    pub async fn execute(&self, request: GetItemRequest) -> Result<GetItemResponse, DomainError> {
        // Find the item
        let item = self
            .item_repository
            .find_by_id(request.id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Item with ID {} not found", request.id))
            })?;

        // Return response
        Ok(GetItemResponse {
            id: item.id,
            sku: item.sku,
            name: item.name,
            description: item.description,
            category: item.category,
            unit: item.unit,
            barcode: item.barcode,
            cost_price: item.cost_price,
            sale_price: item.sale_price,
            reorder_point: item.reorder_point,
            reorder_qty: item.reorder_qty,
            weight: item.weight,
            dimensions: item
                .dimensions
                .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null)),
            metadata: item.metadata,
            active: item.active,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}
