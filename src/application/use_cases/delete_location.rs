use crate::domain::services::location_repository::LocationRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteLocationRequest {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteLocationResponse {
    pub id: Uuid,
    pub active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct DeleteLocationUseCase<R: LocationRepository> {
    location_repository: Arc<R>,
}

impl<R: LocationRepository> DeleteLocationUseCase<R> {
    pub fn new(location_repository: Arc<R>) -> Self {
        Self {
            location_repository,
        }
    }

    pub async fn execute(
        &self,
        request: DeleteLocationRequest,
    ) -> Result<DeleteLocationResponse, DomainError> {
        // Check if location exists
        let mut location = self
            .location_repository
            .find_by_id(request.id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Location with id {} not found", request.id))
            })?;

        // Soft delete by deactivating
        location.deactivate();

        // Update in repository
        self.location_repository.update(&location).await?;

        Ok(DeleteLocationResponse {
            id: location.id,
            active: location.active,
            updated_at: location.updated_at,
        })
    }
}
