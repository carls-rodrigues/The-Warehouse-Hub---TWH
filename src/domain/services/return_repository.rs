use crate::domain::entities::inventory::StockMovement;
use crate::domain::entities::returns::{
    ProcessReturnRequest, Return, ReturnLine,
};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ReturnRepository: Send + Sync {
    async fn create(&self, return_entity: &Return) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<(Return, Vec<ReturnLine>)>, DomainError>;
    async fn find_by_return_number(
        &self,
        return_number: &str,
    ) -> Result<Option<(Return, Vec<ReturnLine>)>, DomainError>;
    async fn update(&self, return_entity: &Return) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Return, Vec<ReturnLine>)>, DomainError>;
    async fn open_return(&self, id: Uuid) -> Result<(Return, Vec<ReturnLine>), DomainError>;
    async fn process_return(
        &self,
        id: Uuid,
        process_request: ProcessReturnRequest,
        created_by: Uuid,
    ) -> Result<(Return, Vec<ReturnLine>, Vec<StockMovement>), DomainError>;
}