use std::sync::Arc;

use crate::domain::services::stock_repository::StockRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct GetTotalQuantityOnHandUseCase<R: StockRepository> {
    stock_repository: Arc<R>,
}

impl<R: StockRepository> GetTotalQuantityOnHandUseCase<R> {
    pub fn new(stock_repository: Arc<R>) -> Self {
        Self { stock_repository }
    }

    pub async fn execute(&self, item_id: uuid::Uuid) -> Result<i32, DomainError> {
        // Validate item_id is not nil
        if item_id.is_nil() {
            return Err(DomainError::ValidationError(
                "Item ID cannot be nil".to_string(),
            ));
        }

        self.stock_repository
            .get_total_quantity_on_hand(item_id)
            .await
    }
}
