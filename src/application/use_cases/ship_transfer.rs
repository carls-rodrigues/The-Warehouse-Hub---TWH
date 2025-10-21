use crate::domain::entities::transfer::{StockMovement, Transfer, TransferLine};
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
pub struct ShipTransferResponse {
    pub transfer: Transfer,
    pub lines: Vec<TransferLine>,
    pub stock_movements: Vec<StockMovement>,
}

pub struct ShipTransferUseCase<T: TransferRepository, D: WebhookDispatcher + 'static> {
    transfer_repo: Arc<T>,
    webhook_dispatcher: Arc<D>,
}

impl<T: TransferRepository, D: WebhookDispatcher + 'static> ShipTransferUseCase<T, D> {
    pub fn new(transfer_repo: Arc<T>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            transfer_repo,
            webhook_dispatcher,
        }
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

        // Dispatch webhook event (non-blocking)
        let webhook_event = WebhookEvent::new(
            WebhookEventType::TransferUpdated,
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
                    "total_quantity": transfer.total_quantity,
                    "notes": transfer.notes,
                    "updated_at": transfer.updated_at,
                    "lines": lines.iter().map(|line| json!({
                        "id": line.id,
                        "item_id": line.item_id,
                        "quantity": line.quantity,
                        "quantity_received": line.quantity_received
                    })).collect::<Vec<_>>()
                },
                "stock_movements": stock_movements.iter().map(|movement| json!({
                    "id": movement.id,
                    "item_id": movement.item_id,
                    "location_id": movement.location_id,
                    "quantity": movement.quantity,
                    "movement_type": match movement.movement_type {
                        crate::domain::entities::transfer::MovementType::Inbound => "INBOUND",
                        crate::domain::entities::transfer::MovementType::Outbound => "OUTBOUND",
                        crate::domain::entities::transfer::MovementType::Adjustment => "ADJUSTMENT",
                        crate::domain::entities::transfer::MovementType::Transfer => "TRANSFER",
                        crate::domain::entities::transfer::MovementType::Initial => "INITIAL",
                    },
                    "reference_type": movement.reference_type.as_str(),
                    "reference_id": movement.reference_id,
                    "reason": movement.reason,
                    "created_by": movement.created_by,
                    "created_at": movement.created_at
                })).collect::<Vec<_>>()
            }),
        );

        // Spawn a task to dispatch the webhook asynchronously
        let dispatcher = Arc::clone(&self.webhook_dispatcher);
        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&webhook_event).await {
                eprintln!("Failed to dispatch transfer updated webhook: {:?}", e);
            }
        });

        Ok(ShipTransferResponse {
            transfer,
            lines,
            stock_movements,
        })
    }
}
