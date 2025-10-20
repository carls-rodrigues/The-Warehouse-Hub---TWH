use crate::domain::entities::transfer::{Transfer, TransferLine};
use crate::domain::services::transfer_repository::TransferRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GetTransferResponse {
    pub transfer: Transfer,
    pub lines: Vec<TransferLine>,
}

pub struct GetTransferUseCase<T: TransferRepository> {
    transfer_repo: T,
}

impl<T: TransferRepository> GetTransferUseCase<T> {
    pub fn new(transfer_repo: T) -> Self {
        Self { transfer_repo }
    }

    pub async fn execute(&self, transfer_id: Uuid) -> Result<GetTransferResponse, DomainError> {
        let (transfer, lines) = self
            .transfer_repo
            .find_by_id(transfer_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Transfer {} not found", transfer_id)))?;

        Ok(GetTransferResponse { transfer, lines })
    }

    pub async fn execute_by_number(
        &self,
        transfer_number: &str,
    ) -> Result<GetTransferResponse, DomainError> {
        let (transfer, lines) = self
            .transfer_repo
            .find_by_transfer_number(transfer_number)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Transfer {} not found", transfer_number))
            })?;

        Ok(GetTransferResponse { transfer, lines })
    }
}
