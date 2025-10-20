use crate::domain::services::item_repository::ItemRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteItemRequest {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteItemResponse {
    pub id: Uuid,
    pub active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct DeleteItemUseCase<R: ItemRepository> {
    item_repository: Arc<R>,
}

impl<R: ItemRepository> DeleteItemUseCase<R> {
    pub fn new(item_repository: Arc<R>) -> Self {
        Self { item_repository }
    }

    pub async fn execute(
        &self,
        request: DeleteItemRequest,
    ) -> Result<DeleteItemResponse, DomainError> {
        // Find the existing item
        let mut item = self
            .item_repository
            .find_by_id(request.id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError(format!("Item with ID {} not found", request.id))
            })?;

        // Check if item is already inactive
        if !item.is_active() {
            return Err(DomainError::ValidationError(format!(
                "Item with ID {} is already deleted",
                request.id
            )));
        }

        // Soft delete by deactivating the item
        item.deactivate();

        // Save to repository
        self.item_repository.update(&item).await?;

        // Return response
        Ok(DeleteItemResponse {
            id: item.id,
            active: item.active,
            updated_at: item.updated_at,
        })
    }
}
