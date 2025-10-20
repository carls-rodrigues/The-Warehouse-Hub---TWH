use std::sync::Arc;

use async_trait::async_trait;
use serde_json;

use crate::domain::entities::inventory::StockMovement;
use crate::domain::entities::item::Item;
use crate::domain::entities::location::Location;
use crate::domain::entities::search::SearchIndexRequest;
use crate::domain::services::{
    item_repository::ItemRepository, location_repository::LocationRepository,
    search_repository::SearchRepository,
};
use crate::shared::error::DomainError;

#[async_trait]
pub trait ProjectionHandler: Send + Sync {
    /// Handle a stock movement and update search indexes
    async fn handle_stock_movement(&self, movement: &StockMovement) -> Result<(), DomainError>;
}

#[derive(Clone)]
pub struct SearchProjectionHandler<SR: SearchRepository, IR: ItemRepository, LR: LocationRepository>
{
    search_repository: Arc<SR>,
    item_repository: Arc<IR>,
    location_repository: Arc<LR>,
}

impl<SR: SearchRepository, IR: ItemRepository, LR: LocationRepository>
    SearchProjectionHandler<SR, IR, LR>
{
    pub fn new(
        search_repository: Arc<SR>,
        item_repository: Arc<IR>,
        location_repository: Arc<LR>,
    ) -> Self {
        Self {
            search_repository,
            item_repository,
            location_repository,
        }
    }

    /// Create searchable content for an item
    fn create_item_search_content(&self, item: &Item) -> String {
        format!(
            "{} {} {} {} {}",
            item.name,
            item.sku,
            item.description.as_deref().unwrap_or(""),
            item.category.as_deref().unwrap_or(""),
            item.unit
        )
    }

    /// Create searchable content for a location
    fn create_location_search_content(&self, location: &Location) -> String {
        format!(
            "{} {} {} {}",
            location.name,
            location.code.as_deref().unwrap_or(""),
            location
                .address
                .as_ref()
                .map(|addr| format!(
                    "{} {} {} {} {}",
                    addr.line1.as_deref().unwrap_or(""),
                    addr.city.as_deref().unwrap_or(""),
                    addr.region.as_deref().unwrap_or(""),
                    addr.postal_code.as_deref().unwrap_or(""),
                    addr.country.as_deref().unwrap_or("")
                ))
                .unwrap_or_default(),
            location.r#type.as_ref().map(|t| t.as_str()).unwrap_or("")
        )
    }

    /// Create searchable content for stock level information
    fn create_stock_level_search_content(
        &self,
        item: &Item,
        location: &Location,
        quantity: i32,
    ) -> String {
        format!(
            "{} {} {} {} {} stock level quantity:{}",
            item.name,
            item.sku,
            location.name,
            location.code.as_deref().unwrap_or(""),
            item.category.as_deref().unwrap_or(""),
            quantity
        )
    }
}

#[async_trait]
impl<SR: SearchRepository, IR: ItemRepository, LR: LocationRepository> ProjectionHandler
    for SearchProjectionHandler<SR, IR, LR>
{
    async fn handle_stock_movement(&self, movement: &StockMovement) -> Result<(), DomainError> {
        // Get the item and location details
        let item = self
            .item_repository
            .find_by_id(movement.item_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Item not found".to_string()))?;

        let location = self
            .location_repository
            .find_by_id(movement.location_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Location not found".to_string()))?;

        // Update item search index
        let item_content = self.create_item_search_content(&item);
        let item_metadata = serde_json::json!({
            "type": "item",
            "sku": item.sku,
            "name": item.name,
            "category": item.category,
            "unit": item.unit,
            "active": item.active
        });

        self.search_repository
            .index_document(SearchIndexRequest {
                entity_type: "item".to_string(),
                entity_id: item.id,
                searchable_content: item_content,
                metadata: Some(item_metadata),
            })
            .await?;

        // Update location search index
        let location_content = self.create_location_search_content(&location);
        let location_metadata = serde_json::json!({
            "type": "location",
            "name": location.name,
            "code": location.code,
            "location_type": location.r#type.as_ref().map(|t| t.as_str()),
            "active": location.active
        });

        self.search_repository
            .index_document(SearchIndexRequest {
                entity_type: "location".to_string(),
                entity_id: location.id,
                searchable_content: location_content,
                metadata: Some(location_metadata),
            })
            .await?;

        // Get current stock level for this item/location combination
        // Note: In a real implementation, you'd get this from the stock repository
        // For now, we'll create a placeholder stock level entry
        let current_quantity = 0; // This should be fetched from stock_levels table

        let stock_content =
            self.create_stock_level_search_content(&item, &location, current_quantity);
        let stock_metadata = serde_json::json!({
            "type": "stock_level",
            "item_id": item.id,
            "item_sku": item.sku,
            "item_name": item.name,
            "location_id": location.id,
            "location_name": location.name,
            "location_code": location.code,
            "quantity": current_quantity,
            "last_movement_id": movement.id,
            "last_movement_type": movement.movement_type.as_str(),
            "last_movement_reference": movement.reference_type.as_str()
        });

        // Use a composite key for stock levels: item_id + location_id
        let stock_entity_id = generate_stock_level_entity_id(item.id, location.id);

        self.search_repository
            .index_document(SearchIndexRequest {
                entity_type: "stock_level".to_string(),
                entity_id: stock_entity_id,
                searchable_content: stock_content,
                metadata: Some(stock_metadata),
            })
            .await?;

        Ok(())
    }
}

/// Generate a deterministic UUID for stock level entities based on item and location IDs
fn generate_stock_level_entity_id(item_id: uuid::Uuid, location_id: uuid::Uuid) -> uuid::Uuid {
    // Create a deterministic UUID by combining item_id and location_id
    // This ensures the same stock level always has the same entity ID
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    item_id.hash(&mut hasher);
    location_id.hash(&mut hasher);
    let hash = hasher.finish();

    // Create a UUID from the hash (not cryptographically secure, but deterministic)
    uuid::Uuid::from_u128(hash as u128)
}
