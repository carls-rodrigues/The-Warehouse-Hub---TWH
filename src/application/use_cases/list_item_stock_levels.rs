use crate::domain::entities::inventory::StockLevelResponse;
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
pub struct ListItemStockLevelsRequest {
    pub item_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemStockLevelsResponse {
    pub item: Option<Item>,
    pub stock_levels: Vec<StockLevelResponse>,
}

pub struct ListItemStockLevelsUseCase<
    SR: StockRepository,
    IR: ItemRepository,
    LR: LocationRepository,
> {
    stock_repository: Arc<SR>,
    item_repository: Arc<IR>,
    location_repository: Arc<LR>,
}

impl<SR: StockRepository, IR: ItemRepository, LR: LocationRepository>
    ListItemStockLevelsUseCase<SR, IR, LR>
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
        request: ListItemStockLevelsRequest,
    ) -> Result<ListItemStockLevelsResponse, DomainError> {
        // Get the item details
        let item = self.item_repository.find_by_id(request.item_id).await?;

        // Get all stock levels for this item
        let stock_levels = self
            .stock_repository
            .get_item_stock_levels(request.item_id)
            .await?;

        // Enrich each stock level with location details
        let mut enriched_levels = Vec::new();
        for level in stock_levels {
            let location = self
                .location_repository
                .find_by_id(level.location_id)
                .await?;

            enriched_levels.push(StockLevelResponse {
                item_id: level.item_id,
                location_id: level.location_id,
                quantity_on_hand: level.quantity_on_hand,
                last_movement_id: level.last_movement_id,
                updated_at: level.updated_at,
                item: item.clone(), // Same item for all levels
                location,
            });
        }

        Ok(ListItemStockLevelsResponse {
            item,
            stock_levels: enriched_levels,
        })
    }
}
