use crate::domain::entities::location::{Location, LocationAddress, LocationType};
use crate::domain::services::location_repository::LocationRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLocationRequest {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLocationResponse {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub address: Option<LocationAddress>,
    pub r#type: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct GetLocationUseCase<R: LocationRepository> {
    location_repository: Arc<R>,
}

impl<R: LocationRepository> GetLocationUseCase<R> {
    pub fn new(location_repository: Arc<R>) -> Self {
        Self {
            location_repository,
        }
    }

    pub async fn execute(
        &self,
        request: GetLocationRequest,
    ) -> Result<GetLocationResponse, DomainError> {
        let location = self
            .location_repository
            .find_by_id(request.id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Location with id {} not found", request.id))
            })?;

        Ok(GetLocationResponse {
            id: location.id,
            name: location.name,
            code: location.code,
            address: location.address,
            r#type: location.r#type.map(|t| t.as_str().to_string()),
            active: location.active,
            created_at: location.created_at,
            updated_at: location.updated_at,
        })
    }
}
