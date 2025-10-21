use crate::domain::entities::inventory::{StockLevel, StockMovement};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PaginatedStockLevels {
    pub items: Vec<StockLevel>,
    pub next_cursor: Option<String>,
}

#[async_trait]
pub trait StockRepository: Send + Sync {
    /// Record a new stock movement and update stock levels atomically
    async fn record_movement(&self, movement: &StockMovement) -> Result<(), DomainError>;

    /// Get stock level for a specific item and location
    async fn get_stock_level(
        &self,
        item_id: Uuid,
        location_id: Uuid,
    ) -> Result<Option<StockLevel>, DomainError>;

    /// Get all stock levels for an item across all locations
    async fn get_item_stock_levels(&self, item_id: Uuid) -> Result<Vec<StockLevel>, DomainError>;

    /// Get all stock levels for a location
    async fn get_location_stock_levels(
        &self,
        location_id: Uuid,
    ) -> Result<Vec<StockLevel>, DomainError>;

    /// Get stock movements for an item with pagination
    async fn get_item_movements(
        &self,
        item_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovement>, DomainError>;

    /// Get stock movements for a location with pagination
    async fn get_location_movements(
        &self,
        location_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovement>, DomainError>;

    /// Get stock movements for a specific item at a specific location
    async fn get_stock_movements(
        &self,
        item_id: Uuid,
        location_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovement>, DomainError>;

    /// Get a specific stock movement by ID
    async fn get_movement_by_id(&self, id: Uuid) -> Result<Option<StockMovement>, DomainError>;

    /// Get total quantity on hand for an item across all locations
    async fn get_total_quantity_on_hand(&self, item_id: Uuid) -> Result<i32, DomainError>;

    /// Initialize stock level for a new item/location combination
    async fn initialize_stock_level(
        &self,
        item_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError>;

    /// Check if stock level exists for item/location combination
    async fn stock_level_exists(
        &self,
        item_id: Uuid,
        location_id: Uuid,
    ) -> Result<bool, DomainError>;

    /// Get stock levels below a threshold for low stock report
    async fn get_stock_levels_below_threshold(
        &self,
        threshold: i32,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<PaginatedStockLevels, DomainError>;

    /// Get stock levels by location with pagination
    async fn get_stock_levels_by_location(
        &self,
        location_id: Uuid,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<PaginatedStockLevels, DomainError>;

    /// Get all stock levels with pagination
    async fn get_all_stock_levels(
        &self,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<PaginatedStockLevels, DomainError>;
}
