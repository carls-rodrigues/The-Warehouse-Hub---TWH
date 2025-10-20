use crate::domain::entities::transfer::{
    ReceiveTransferRequest, StockMovement, Transfer, TransferLine,
};
use crate::domain::services::transfer_repository::TransferRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ReceiveTransferResponse {
    pub transfer: Transfer,
    pub lines: Vec<TransferLine>,
    pub stock_movements: Vec<StockMovement>,
}

pub struct ReceiveTransferUseCase<T: TransferRepository> {
    transfer_repo: T,
}

impl<T: TransferRepository> ReceiveTransferUseCase<T> {
    pub fn new(transfer_repo: T) -> Self {
        Self { transfer_repo }
    }

    pub async fn execute(
        &self,
        transfer_id: Uuid,
        request: ReceiveTransferRequest,
        created_by: Uuid,
    ) -> Result<ReceiveTransferResponse, DomainError> {
        // Receive the transfer through the repository
        let (transfer, lines, stock_movements) = self
            .transfer_repo
            .receive_transfer(transfer_id, request.lines, created_by)
            .await?;

        Ok(ReceiveTransferResponse {
            transfer,
            lines,
            stock_movements,
        })
    }
}
