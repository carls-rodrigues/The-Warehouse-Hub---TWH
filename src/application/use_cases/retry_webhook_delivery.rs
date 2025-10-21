use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use std::sync::Arc;
use uuid::Uuid;

pub struct RetryWebhookDeliveryUseCase<R: WebhookRepository, D: WebhookDispatcher> {
    webhook_dispatcher: Arc<D>,
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository, D: WebhookDispatcher> RetryWebhookDeliveryUseCase<R, D> {
    pub fn new(webhook_dispatcher: Arc<D>, webhook_repository: Arc<R>) -> Self {
        Self {
            webhook_dispatcher,
            webhook_repository,
        }
    }

    pub async fn execute(
        &self,
        delivery_id: Uuid,
        user_id: Uuid,
    ) -> Result<RetryWebhookDeliveryResponse, DomainError> {
        // Get the delivery
        let delivery = self
            .webhook_repository
            .get_delivery(delivery_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Delivery {} not found", delivery_id)))?;

        // Verify webhook ownership
        let webhook = self
            .webhook_repository
            .get_webhook(delivery.webhook_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Webhook {} not found", delivery.webhook_id))
            })?;

        if webhook.created_by != user_id {
            return Err(DomainError::BusinessLogicError(
                "You can only retry deliveries for your own webhooks".to_string(),
            ));
        }

        // Check if delivery can be retried
        if delivery.attempt_count >= 5 {
            return Err(DomainError::ValidationError(
                "Delivery has exceeded maximum retry attempts".to_string(),
            ));
        }

        if delivery.status.as_str() == "SUCCESS" {
            return Err(DomainError::ValidationError(
                "Cannot retry a successful delivery".to_string(),
            ));
        }

        // Retry the delivery
        match self.webhook_dispatcher.retry_delivery(delivery_id).await {
            Ok(_) => Ok(RetryWebhookDeliveryResponse {
                success: true,
                message: "Webhook delivery retry initiated successfully".to_string(),
            }),
            Err(e) => Ok(RetryWebhookDeliveryResponse {
                success: false,
                message: format!("Failed to retry webhook delivery: {}", e),
            }),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RetryWebhookDeliveryResponse {
    pub success: bool,
    pub message: String,
}
