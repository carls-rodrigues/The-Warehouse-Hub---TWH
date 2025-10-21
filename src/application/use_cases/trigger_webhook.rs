use crate::domain::entities::webhook::WebhookEvent;
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerWebhookRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
}

pub struct TriggerWebhookUseCase<D: WebhookDispatcher> {
    webhook_dispatcher: Arc<D>,
}

impl<D: WebhookDispatcher> TriggerWebhookUseCase<D> {
    pub fn new(webhook_dispatcher: Arc<D>) -> Self {
        Self { webhook_dispatcher }
    }

    pub async fn execute(&self, request: TriggerWebhookRequest) -> Result<(), DomainError> {
        // Parse the event type from string
        let event_type = serde_json::from_value(serde_json::json!(request.event_type))
            .map_err(|e| DomainError::ValidationError(format!("Invalid event type: {}", e)))?;

        // Create webhook event
        let event = WebhookEvent::new(event_type, request.payload);

        // Dispatch the event to all subscribed webhooks
        self.webhook_dispatcher.dispatch_event(&event).await?;

        Ok(())
    }
}
