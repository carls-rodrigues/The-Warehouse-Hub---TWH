use crate::domain::entities::returns::{CreateReturnRequest, Return, ReturnLine};
use crate::domain::entities::webhook::{WebhookEvent, WebhookEventType};
use crate::domain::services::return_repository::ReturnRepository;
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::shared::error::DomainError;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateReturnResponse {
    pub return_entity: Return,
}

pub struct CreateReturnUseCase<R: ReturnRepository, D: WebhookDispatcher + 'static> {
    return_repository: Arc<R>,
    webhook_dispatcher: Arc<D>,
}

impl<R: ReturnRepository, D: WebhookDispatcher + 'static> CreateReturnUseCase<R, D> {
    pub fn new(return_repository: Arc<R>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            return_repository,
            webhook_dispatcher,
        }
    }

    pub async fn execute(
        &self,
        request: CreateReturnRequest,
        created_by: Uuid,
    ) -> Result<CreateReturnResponse, DomainError> {
        // Validate request
        if request.lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Return must have at least one line".to_string(),
            ));
        }

        // Generate return number (in a real app, this might come from a sequence)
        let return_number = format!("RT-{}", Uuid::new_v4().simple());

        // Create return
        let mut return_entity = Return::new(
            return_number,
            request.customer_id,
            request.location_id,
            created_by,
        )?;

        // Set notes if provided
        return_entity.notes = request.notes;

        // Add lines
        for line_req in request.lines {
            let line = ReturnLine::new(
                return_entity.id,
                line_req.item_id,
                line_req.quantity,
                line_req.unit_price,
                line_req.reason,
            )?;
            return_entity.add_line(line)?;
        }

        // Create in repository
        self.return_repository.create(&return_entity).await?;

        // Dispatch webhook event (non-blocking)
        let webhook_event = WebhookEvent::new(
            WebhookEventType::ReturnCreated,
            json!({
                "return": {
                    "id": return_entity.id,
                    "return_number": return_entity.return_number,
                    "customer_id": return_entity.customer_id,
                    "location_id": return_entity.location_id,
                    "status": match return_entity.status {
                        crate::domain::entities::returns::ReturnStatus::Draft => "DRAFT",
                        crate::domain::entities::returns::ReturnStatus::Open => "OPEN",
                        crate::domain::entities::returns::ReturnStatus::Received => "RECEIVED",
                        crate::domain::entities::returns::ReturnStatus::Cancelled => "CANCELLED",
                    },
                    "total_quantity": return_entity.total_quantity,
                    "notes": return_entity.notes,
                    "created_at": return_entity.created_at,
                    "lines": return_entity.lines.iter().map(|line| json!({
                        "id": line.id,
                        "item_id": line.item_id,
                        "quantity": line.quantity,
                        "quantity_received": line.quantity_received,
                        "unit_price": line.unit_price,
                        "reason": line.reason
                    })).collect::<Vec<_>>()
                }
            }),
        );

        // Spawn a task to dispatch the webhook asynchronously
        let dispatcher = Arc::clone(&self.webhook_dispatcher);
        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&webhook_event).await {
                eprintln!("Failed to dispatch return created webhook: {:?}", e);
            }
        });

        Ok(CreateReturnResponse { return_entity })
    }
}
