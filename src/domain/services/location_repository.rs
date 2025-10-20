use crate::domain::entities::location::{Location, UpdateLocationRequest};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LocationRepository: Send + Sync {
    /// Find a location by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Location>, DomainError>;

    /// Find a location by its code
    async fn find_by_code(&self, code: &str) -> Result<Option<Location>, DomainError>;

    /// Save a new location
    async fn save(&self, location: &Location) -> Result<(), DomainError>;

    /// Update an existing location
    async fn update(&self, location: &Location) -> Result<(), DomainError>;

    /// Delete a location by ID
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// List all locations with pagination
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Location>, DomainError>;

    /// Count total locations
    async fn count(&self) -> Result<i64, DomainError>;

    /// Check if code is already taken by another location
    async fn code_exists(
        &self,
        code: &str,
        exclude_location_id: Option<Uuid>,
    ) -> Result<bool, DomainError>;
}
