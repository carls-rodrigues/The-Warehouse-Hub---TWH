use crate::domain::entities::transfer::{StockMovement, Transfer, TransferLine};
use crate::domain::services::transfer_repository::TransferRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ShipTransferResponse {
    pub transfer: Transfer,
    pub lines: Vec<TransferLine>,
    pub stock_movements: Vec<StockMovement>,
}

pub struct ShipTransferUseCase<T: TransferRepository> {
    transfer_repo: T,
}

impl<T: TransferRepository> ShipTransferUseCase<T> {
    pub fn new(transfer_repo: T) -> Self {
        Self { transfer_repo }
    }

    pub async fn execute(
        &self,
        transfer_id: Uuid,
        created_by: Uuid,
    ) -> Result<ShipTransferResponse, DomainError> {
        // Ship the transfer through the repository
        let (transfer, lines, stock_movements) = self
            .transfer_repo
            .ship_transfer(transfer_id, created_by)
            .await?;

        Ok(ShipTransferResponse {
            transfer,
            lines,
            stock_movements,
        })
    }
}
