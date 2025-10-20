use crate::domain::entities::location::{
    Location, LocationAddress, LocationType, UpdateLocationRequest,
};
use crate::domain::services::location_repository::LocationRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLocationRequestDto {
    pub name: Option<String>,
    pub code: Option<String>,
    pub address: Option<LocationAddress>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLocationResponse {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
    pub active: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub etag: String,
}

pub struct UpdateLocationUseCase<R: LocationRepository> {
    location_repository: Arc<R>,
}

impl<R: LocationRepository> UpdateLocationUseCase<R> {
    pub fn new(location_repository: Arc<R>) -> Self {
        Self {
            location_repository,
        }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        request: UpdateLocationRequestDto,
    ) -> Result<UpdateLocationResponse, DomainError> {
        // Find the existing location
        let mut location = self
            .location_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Location with id {} not found", id)))?;

        // Check if code is being changed and if it conflicts
        if let Some(ref new_code) = request.code {
            if location.code.as_ref() != Some(new_code) {
                let code_exists = self
                    .location_repository
                    .code_exists(new_code, Some(id))
                    .await?;
                if code_exists {
                    return Err(DomainError::ValidationError(format!(
                        "Location with code '{}' already exists",
                        new_code
                    )));
                }
            }
        }

        // Update the location
        let update_request = UpdateLocationRequest {
            name: request.name,
            code: request.code,
            address: request.address,
            r#type: request.r#type,
        };

        location.update(update_request)?;

        // Save to repository
        self.location_repository.update(&location).await?;

        // Generate ETag from updated_at timestamp
        let etag = format!("\"{}\"", location.updated_at.timestamp());

        Ok(UpdateLocationResponse {
            id: location.id,
            name: location.name,
            code: location.code,
            r#type: location.r#type.map(|t| t.as_str().to_string()),
            active: location.active,
            updated_at: location.updated_at,
            etag,
        })
    }
}
