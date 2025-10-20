use std::sync::Arc;

use crate::domain::entities::inventory::{StockMovement, StockMovementResponse};
use crate::domain::entities::item::Item;
use crate::domain::entities::location::Location;
use crate::domain::services::item_repository::ItemRepository;
use crate::domain::services::location_repository::LocationRepository;
use crate::domain::services::stock_repository::StockRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct GetStockMovementsUseCase<SR: StockRepository, IR: ItemRepository, LR: LocationRepository>
{
    stock_repository: Arc<SR>,
    item_repository: Arc<IR>,
    location_repository: Arc<LR>,
}

impl<SR: StockRepository, IR: ItemRepository, LR: LocationRepository>
    GetStockMovementsUseCase<SR, IR, LR>
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
        item_id: Option<uuid::Uuid>,
        location_id: Option<uuid::Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovementResponse>, DomainError> {
        // Validate pagination parameters
        if limit <= 0 || limit > 1000 {
            return Err(DomainError::ValidationError(
                "Limit must be between 1 and 1000".to_string(),
            ));
        }
        if offset < 0 {
            return Err(DomainError::ValidationError(
                "Offset must be non-negative".to_string(),
            ));
        }

        // Get stock movements based on filters
        let movements = match (item_id, location_id) {
            (Some(item_id), Some(location_id)) => {
                self.stock_repository
                    .get_stock_movements(item_id, location_id, limit, offset)
                    .await?
            }
            (Some(item_id), None) => {
                self.stock_repository
                    .get_item_movements(item_id, limit, offset)
                    .await?
            }
            (None, Some(location_id)) => {
                self.stock_repository
                    .get_location_movements(location_id, limit, offset)
                    .await?
            }
            (None, None) => {
                return Err(DomainError::ValidationError(
                    "Either item_id or location_id must be provided".to_string(),
                ));
            }
        };

        // Enrich movements with item and location data
        let mut enriched_movements = Vec::new();

        for movement in movements {
            // Get item details
            let item = self
                .item_repository
                .find_by_id(movement.item_id)
                .await?
                .ok_or_else(|| DomainError::NotFound("Item not found".to_string()))?;

            // Get location details
            let location = self
                .location_repository
                .find_by_id(movement.location_id)
                .await?
                .ok_or_else(|| DomainError::NotFound("Location not found".to_string()))?;

            enriched_movements.push(StockMovementResponse {
                id: movement.id,
                item_id: movement.item_id,
                location_id: movement.location_id,
                movement_type: movement.movement_type.as_str().to_string(),
                reference_type: movement.reference_type.as_str().to_string(),
                reference_id: movement.reference_id,
                quantity: movement.quantity,
                reason: movement.reason,
                created_at: movement.created_at,
                created_by: movement.created_by,
                item: Some(item),
                location: Some(location),
                created_by_user: None, // TODO: Implement user lookup when needed
            });
        }

        Ok(enriched_movements)
    }
}
