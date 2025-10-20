use crate::domain::entities::location::{Location, LocationAddress, LocationType};
use crate::domain::services::location_repository::LocationRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListLocationsRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationSummary {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListLocationsResponse {
    pub locations: Vec<LocationSummary>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

pub struct ListLocationsUseCase<R: LocationRepository> {
    location_repository: Arc<R>,
}

impl<R: LocationRepository> ListLocationsUseCase<R> {
    pub fn new(location_repository: Arc<R>) -> Self {
        Self {
            location_repository,
        }
    }

    pub async fn execute(
        &self,
        request: ListLocationsRequest,
    ) -> Result<ListLocationsResponse, DomainError> {
        let limit = request.limit.unwrap_or(25).min(200);
        let offset = request.offset.unwrap_or(0);

        let locations = self.location_repository.list(limit, offset).await?;
        let total_count = self.location_repository.count().await?;

        let locations_dto = locations
            .into_iter()
            .map(|location| LocationSummary {
                id: location.id.to_string(),
                name: location.name,
                code: location.code,
                r#type: location.r#type.map(|t| t.as_str().to_string()),
                active: location.active,
                created_at: location.created_at.to_rfc3339(),
                updated_at: location.updated_at.to_rfc3339(),
            })
            .collect();

        Ok(ListLocationsResponse {
            locations: locations_dto,
            total_count,
            limit,
            offset,
        })
    }
}
