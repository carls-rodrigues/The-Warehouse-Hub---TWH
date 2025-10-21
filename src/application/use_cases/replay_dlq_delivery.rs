use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub struct ReplayDlqDeliveryUseCase<R: WebhookRepository, D: WebhookDispatcher> {
    webhook_dispatcher: Arc<D>,
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository, D: WebhookDispatcher> ReplayDlqDeliveryUseCase<R, D> {
    pub fn new(webhook_dispatcher: Arc<D>, webhook_repository: Arc<R>) -> Self {
        Self {
            webhook_dispatcher,
            webhook_repository,
        }
    }

    pub async fn execute(
        &self,
        delivery_id: Uuid,
    ) -> Result<ReplayDlqDeliveryResponse, DomainError> {
        // Get the delivery
        let mut delivery = self
            .webhook_repository
            .get_delivery(delivery_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Delivery {} not found", delivery_id)))?;

        // Check if delivery is in DLQ
        if delivery.status != crate::domain::entities::webhook::DeliveryStatus::Dlq {
            return Err(DomainError::ValidationError(
                "Delivery is not in DLQ status".to_string(),
            ));
        }

        // Get the webhook to get the URL and secret
        let webhook = self
            .webhook_repository
            .get_webhook(delivery.webhook_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Webhook {} not found", delivery.webhook_id))
            })?;

        // Attempt to redeliver using the retry_delivery method
        let result = self.webhook_dispatcher.retry_delivery(delivery_id).await;

        // Update delivery status based on result
        match result {
            Ok(_) => {
                delivery.status = crate::domain::entities::webhook::DeliveryStatus::Success;
                delivery.record_attempt(true, Some(200), None, None);
                self.webhook_repository.update_delivery(&delivery).await?;

                Ok(ReplayDlqDeliveryResponse {
                    success: true,
                    message: "Delivery replayed successfully".to_string(),
                    new_status: delivery.status.as_str().to_string(),
                })
            }
            Err(e) => {
                delivery.record_attempt(false, None, None, Some(e.to_string()));
                // Keep in DLQ status but increment attempt count
                self.webhook_repository.update_delivery(&delivery).await?;

                Ok(ReplayDlqDeliveryResponse {
                    success: false,
                    message: format!("Delivery replay failed: {}", e),
                    new_status: delivery.status.as_str().to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReplayDlqDeliveryResponse {
    pub success: bool,
    pub message: String,
    pub new_status: String,
}
