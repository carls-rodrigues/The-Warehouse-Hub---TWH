use crate::domain::entities::inventory::{StockLevel, StockLevelResponse};
use crate::domain::entities::item::Item;
use crate::domain::entities::location::Location;
use crate::domain::services::item_repository::ItemRepository;
use crate::domain::services::location_repository::LocationRepository;
use crate::domain::services::stock_repository::StockRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetStockLevelRequest {
    pub item_id: Uuid,
    pub location_id: Uuid,
}

pub struct GetStockLevelUseCase<SR: StockRepository, IR: ItemRepository, LR: LocationRepository> {
    stock_repository: Arc<SR>,
    item_repository: Arc<IR>,
    location_repository: Arc<LR>,
}

impl<SR: StockRepository, IR: ItemRepository, LR: LocationRepository>
    GetStockLevelUseCase<SR, IR, LR>
{
    pub fn new(
        stock_repository: Arc<SR>,
        item_repository: Arc<IR>,
        location_repository: Arc<LR>,
    ) -> Self {
        Self {
            stock_repository,
            item_repository,
            location_repository,
        }
    }

    pub async fn execute(
        &self,
        request: GetStockLevelRequest,
    ) -> Result<Option<StockLevelResponse>, DomainError> {
        // Get the stock level
        let stock_level = match self
            .stock_repository
            .get_stock_level(request.item_id, request.location_id)
            .await?
        {
            Some(level) => level,
            None => return Ok(None), // No stock level exists for this item/location
        };

        // Get the item details
        let item = self.item_repository.find_by_id(request.item_id).await?;

        // Get the location details
        let location = self
            .location_repository
            .find_by_id(request.location_id)
            .await?;

        Ok(Some(StockLevelResponse {
            item_id: stock_level.item_id,
            location_id: stock_level.location_id,
            quantity_on_hand: stock_level.quantity_on_hand,
            last_movement_id: stock_level.last_movement_id,
            updated_at: stock_level.updated_at,
            item,
            location,
        }))
    }
}
