use crate::domain::entities::item::{Item, UpdateItemRequest as DomainUpdateRequest};
use crate::domain::services::item_repository::ItemRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateItemRequest {
    pub id: Uuid,
    pub sku: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub barcode: Option<String>,
    pub cost_price: Option<f64>,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub if_match: Option<String>, // ETag for optimistic concurrency
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateItemResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub cost_price: f64,
    pub active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub etag: String, // New ETag for the updated item
}

pub struct UpdateItemUseCase<R: ItemRepository> {
    item_repository: Arc<R>,
}

impl<R: ItemRepository> UpdateItemUseCase<R> {
    pub fn new(item_repository: Arc<R>) -> Self {
        Self { item_repository }
    }

    pub async fn execute(
        &self,
        request: UpdateItemRequest,
    ) -> Result<UpdateItemResponse, DomainError> {
        // Find the existing item
        let mut item = self
            .item_repository
            .find_by_id(request.id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Item with ID {} not found", request.id))
            })?;

        // Check optimistic concurrency if If-Match header is provided
        if let Some(if_match) = &request.if_match {
            let current_etag = Self::generate_etag(&item);
            if &current_etag != if_match {
                return Err(DomainError::ValidationError(
                    "ETag mismatch: item has been modified by another request".to_string(),
                ));
            }
        }

        // Check SKU uniqueness if SKU is being updated
        if let Some(ref new_sku) = request.sku {
            if new_sku != &item.sku {
                let sku_exists = self
                    .item_repository
                    .sku_exists(new_sku, Some(item.id))
                    .await?;
                if sku_exists {
                    return Err(DomainError::ValidationError(format!(
                        "Item with SKU '{}' already exists",
                        new_sku
                    )));
                }
            }
        }

        // Parse dimensions if provided
        let dimensions = if let Some(dimensions_json) = request.dimensions {
            Some(serde_json::from_value(dimensions_json).map_err(|_| {
                DomainError::ValidationError("Invalid dimensions format".to_string())
            })?)
        } else {
            None // Don't move item.dimensions yet
        };

        // Create update request
        let update_request = DomainUpdateRequest {
            sku: request.sku,
            name: request.name,
            description: request.description,
            category: request.category,
            unit: request.unit,
            barcode: request.barcode,
            cost_price: request.cost_price,
            sale_price: request.sale_price,
            reorder_point: request.reorder_point,
            reorder_qty: request.reorder_qty,
            weight: request.weight,
            dimensions,
            metadata: request.metadata,
        };

        // Update the item
        item.update(update_request)?;

        // Save to repository
        self.item_repository.update(&item).await?;

        // Generate new ETag
        let etag = Self::generate_etag(&item);

        // Return response
        Ok(UpdateItemResponse {
            id: item.id,
            sku: item.sku,
            name: item.name,
            unit: item.unit,
            cost_price: item.cost_price,
            active: item.active,
            updated_at: item.updated_at,
            etag,
        })
    }

    // Generate ETag based on item ID and updated_at timestamp
    fn generate_etag(item: &Item) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        item.id.hash(&mut hasher);
        item.updated_at.hash(&mut hasher);
        format!("\"{:x}\"", hasher.finish())
    }
}
