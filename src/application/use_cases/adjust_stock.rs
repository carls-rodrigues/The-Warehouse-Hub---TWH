use crate::domain::entities::inventory::{
    Adjustment, AdjustmentReason, MovementType, ReferenceType, StockAdjustmentRequest,
    StockMovement,
};
use crate::domain::services::stock_repository::StockRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustStockResponse {
    pub adjustment: Adjustment,
    pub new_quantity_on_hand: i32,
}

pub struct AdjustStockUseCase<R: StockRepository> {
    stock_repository: Arc<R>,
}

impl<R: StockRepository> AdjustStockUseCase<R> {
    pub fn new(stock_repository: Arc<R>) -> Self {
        Self { stock_repository }
    }

    pub async fn execute(
        &self,
        request: StockAdjustmentRequest,
        created_by: Uuid,
    ) -> Result<AdjustStockResponse, DomainError> {
        // Create the stock movement
        let movement = StockMovement::new(
            request.item_id,
            request.location_id,
            MovementType::Adjustment,
            request.qty_change,
            ReferenceType::Adjustment,
            None, // No reference ID for adjustments
            Some(request.reason.as_str().to_string()),
            Some(created_by),
        )?;

        // Record the movement (this will update stock levels atomically)
        self.stock_repository.record_movement(&movement).await?;

        // Get the updated stock level
        let stock_level = self
            .stock_repository
            .get_stock_level(request.item_id, request.location_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound("Stock level not found after adjustment".to_string())
            })?;

        let adjustment = Adjustment {
            id: movement.id,
            item_id: request.item_id,
            location_id: request.location_id,
            qty_change: request.qty_change,
            reason: request.reason,
            note: request.note,
            created_by,
            created_at: movement.created_at,
        };

        Ok(AdjustStockResponse {
            adjustment,
            new_quantity_on_hand: stock_level.quantity_on_hand,
        })
    }
}
