use crate::domain::entities::transfer::{
    CreateTransferRequest, ReceiveTransferRequest, StockMovement, Transfer, TransferLine,
};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TransferRepository: Send + Sync {
    async fn create(&self, transfer: &Transfer) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<(Transfer, Vec<TransferLine>)>, DomainError>;
    async fn find_by_transfer_number(
        &self,
        transfer_number: &str,
    ) -> Result<Option<(Transfer, Vec<TransferLine>)>, DomainError>;
    async fn update(&self, transfer: &Transfer) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Transfer, Vec<TransferLine>)>, DomainError>;
    async fn ship_transfer(
        &self,
        id: Uuid,
        created_by: Uuid,
    ) -> Result<(Transfer, Vec<TransferLine>, Vec<StockMovement>), DomainError>;
    async fn receive_transfer(
        &self,
        id: Uuid,
        received_lines: Vec<crate::domain::entities::transfer::ReceiveTransferLineRequest>,
        created_by: Uuid,
    ) -> Result<(Transfer, Vec<TransferLine>, Vec<StockMovement>), DomainError>;
}
