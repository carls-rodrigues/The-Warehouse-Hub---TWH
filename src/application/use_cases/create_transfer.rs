use crate::domain::entities::transfer::{
    CreateTransferRequest, StockMovement, Transfer, TransferLine,
};
use crate::domain::services::transfer_repository::TransferRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateTransferResponse {
    pub transfer: Transfer,
}

pub struct CreateTransferUseCase<T: TransferRepository> {
    transfer_repo: T,
}

impl<T: TransferRepository> CreateTransferUseCase<T> {
    pub fn new(transfer_repo: T) -> Self {
        Self { transfer_repo }
    }

    pub async fn execute(
        &self,
        request: CreateTransferRequest,
        created_by: Uuid,
    ) -> Result<CreateTransferResponse, DomainError> {
        // Validate request
        if request.lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Transfer must have at least one line".to_string(),
            ));
        }

        // Generate transfer number (in a real app, this might come from a sequence)
        let transfer_number = format!("TR-{}", Uuid::new_v4().simple());

        // Create transfer
        let mut transfer = Transfer::new(
            transfer_number,
            request.from_location_id,
            request.to_location_id,
            created_by,
        )?;

        // Set notes if provided
        transfer.notes = request.notes;

        // Add lines
        for line_req in request.lines {
            let line = TransferLine::new(transfer.id, line_req.item_id, line_req.quantity)?;
            transfer.add_line(line)?;
        }

        // Open the transfer (moves from Draft to Open)
        transfer.open()?;

        // Create in repository
        self.transfer_repo.create(&transfer).await?;

        Ok(CreateTransferResponse { transfer })
    }
}
