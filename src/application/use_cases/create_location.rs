use crate::domain::entities::location::{
    Location, LocationAddress, LocationType, UpdateLocationRequest,
};
use crate::domain::services::location_repository::LocationRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLocationRequest {
    pub name: String,
    pub code: Option<String>,
    pub address: Option<LocationAddress>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLocationResponse {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct CreateLocationUseCase<R: LocationRepository> {
    location_repository: Arc<R>,
}

impl<R: LocationRepository> CreateLocationUseCase<R> {
    pub fn new(location_repository: Arc<R>) -> Self {
        Self {
            location_repository,
        }
    }

    pub async fn execute(
        &self,
        request: CreateLocationRequest,
    ) -> Result<CreateLocationResponse, DomainError> {
        // Check if code already exists (if provided)
        if let Some(ref code) = request.code {
            let code_exists = self.location_repository.code_exists(code, None).await?;
            if code_exists {
                return Err(DomainError::ValidationError(format!(
                    "Location with code '{}' already exists",
                    code
                )));
            }
        }

        // Create the location with required fields
        let mut location = Location::new(request.name)?;

        // Update with optional fields
        let update_request = UpdateLocationRequest {
            name: None, // Name is already set
            code: request.code,
            address: request.address,
            r#type: request.r#type,
        };

        location.update(update_request)?;

        // Save to repository
        self.location_repository.save(&location).await?;

        // Return response
        Ok(CreateLocationResponse {
            id: location.id,
            name: location.name,
            code: location.code,
            r#type: location.r#type.map(|t| t.as_str().to_string()),
            active: location.active,
            created_at: location.created_at,
            updated_at: location.updated_at,
        })
    }
}
