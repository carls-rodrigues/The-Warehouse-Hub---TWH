use crate::domain::entities::webhook::{WebhookEvent, WebhookEventType};
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestWebhookUseCase<R: WebhookRepository, D: WebhookDispatcher> {
    webhook_repository: Arc<R>,
    webhook_dispatcher: Arc<D>,
}

impl<R: WebhookRepository, D: WebhookDispatcher> TestWebhookUseCase<R, D> {
    pub fn new(webhook_repository: Arc<R>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            webhook_repository,
            webhook_dispatcher,
        }
    }

    pub async fn execute(
        &self,
        webhook_id: Uuid,
        user_id: Uuid,
    ) -> Result<TestWebhookResponse, DomainError> {
        // Verify webhook ownership
        let webhook = self
            .webhook_repository
            .get_webhook(webhook_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Webhook {} not found", webhook_id)))?;

        if webhook.created_by != user_id {
            return Err(DomainError::BusinessLogicError(
                "You can only test your own webhooks".to_string(),
            ));
        }

        // Create a test event
        let test_event = WebhookEvent {
            id: Uuid::new_v4(),
            event_type: WebhookEventType::StockMovement,
            payload: serde_json::json!({
                "test": true,
                "message": "This is a test webhook delivery",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "webhook_id": webhook_id
            }),
            created_at: chrono::Utc::now(),
        };

        // Store the test event
        self.webhook_repository.create_event(&test_event).await?;

        // Create a delivery for this webhook
        let delivery =
            crate::domain::entities::webhook::WebhookDelivery::new(webhook.id, test_event.id);
        self.webhook_repository.create_delivery(&delivery).await?;

        // Dispatch the test delivery
        let result = self.webhook_dispatcher.retry_delivery(delivery.id).await;

        match result {
            Ok(_) => {
                // Get the delivery details to return response info
                let delivery = self
                    .webhook_repository
                    .get_delivery(delivery.id)
                    .await?
                    .ok_or_else(|| {
                        DomainError::NotFound(format!("Delivery {} not found", delivery.id))
                    })?;

                Ok(TestWebhookResponse {
                    success: true,
                    message: "Test webhook delivered successfully".to_string(),
                    delivery_id: Some(delivery.id),
                    response_status: delivery.response_status,
                    response_body: delivery.response_body,
                    error_message: delivery.error_message,
                })
            }
            Err(e) => Ok(TestWebhookResponse {
                success: false,
                message: format!("Test webhook delivery failed: {}", e),
                delivery_id: Some(delivery.id),
                response_status: None,
                response_body: None,
                error_message: Some(e.to_string()),
            }),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct TestWebhookResponse {
    pub success: bool,
    pub message: String,
    pub delivery_id: Option<Uuid>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
}
