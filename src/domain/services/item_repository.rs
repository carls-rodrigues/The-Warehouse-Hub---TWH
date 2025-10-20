use crate::domain::entities::item::{Item, UpdateItemRequest};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ItemRepository: Send + Sync {
    /// Find an item by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Item>, DomainError>;

    /// Find an item by its SKU
    async fn find_by_sku(&self, sku: &str) -> Result<Option<Item>, DomainError>;

    /// Save a new item
    async fn save(&self, item: &Item) -> Result<(), DomainError>;

    /// Update an existing item
    async fn update(&self, item: &Item) -> Result<(), DomainError>;

    /// Delete an item by ID
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// List all items with pagination
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Item>, DomainError>;

    /// Count total items
    async fn count(&self) -> Result<i64, DomainError>;

    /// Check if SKU is already taken by another item
    async fn sku_exists(
        &self,
        sku: &str,
        exclude_item_id: Option<Uuid>,
    ) -> Result<bool, DomainError>;
}
