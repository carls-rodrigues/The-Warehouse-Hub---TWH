use crate::domain::entities::transfer::{
    CreateTransferRequest, StockMovement, Transfer, TransferLine,
};
use crate::domain::entities::webhook::{WebhookEvent, WebhookEventType};
use crate::domain::services::transfer_repository::TransferRepository;
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateTransferResponse {
    pub transfer: Transfer,
}

pub struct CreateTransferUseCase<T: TransferRepository, D: WebhookDispatcher + 'static> {
    transfer_repo: Arc<T>,
    webhook_dispatcher: Arc<D>,
}

impl<T: TransferRepository, D: WebhookDispatcher + 'static> CreateTransferUseCase<T, D> {
    pub fn new(transfer_repo: Arc<T>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            transfer_repo,
            webhook_dispatcher,
        }
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

        // Dispatch webhook event (non-blocking)
        let webhook_event = WebhookEvent::new(
            WebhookEventType::TransferCreated,
            json!({
                "transfer": {
                    "id": transfer.id,
                    "transfer_number": transfer.transfer_number,
                    "from_location_id": transfer.from_location_id,
                    "to_location_id": transfer.to_location_id,
                    "status": match transfer.status {
                        crate::domain::entities::transfer::TransferStatus::Draft => "DRAFT",
                        crate::domain::entities::transfer::TransferStatus::Open => "OPEN",
                        crate::domain::entities::transfer::TransferStatus::InTransit => "IN_TRANSIT",
                        crate::domain::entities::transfer::TransferStatus::Received => "RECEIVED",
                        crate::domain::entities::transfer::TransferStatus::Cancelled => "CANCELLED",
                    },
                    "notes": transfer.notes,
                    "created_at": transfer.created_at,
                    "lines": transfer.lines.iter().map(|line| json!({
                        "id": line.id,
                        "item_id": line.item_id,
                        "quantity": line.quantity
                    })).collect::<Vec<_>>()
                }
            }),
        );

        // Spawn a task to dispatch the webhook asynchronously
        let dispatcher = Arc::clone(&self.webhook_dispatcher);
        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&webhook_event).await {
                eprintln!("Failed to dispatch transfer created webhook: {:?}", e);
            }
        });

        Ok(CreateTransferResponse { transfer })
    }
}
