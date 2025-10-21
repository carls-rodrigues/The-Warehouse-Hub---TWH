use crate::domain::entities::webhook::{Webhook, WebhookDelivery, WebhookEvent, WebhookEventType};
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait WebhookDispatcher: Send + Sync {
    /// Dispatch a webhook event to all subscribed webhooks
    async fn dispatch_event(&self, event: &WebhookEvent) -> Result<(), DomainError>;

    /// Retry a specific delivery
    async fn retry_delivery(&self, delivery_id: Uuid) -> Result<(), DomainError>;

    /// Process pending deliveries (for background job)
    async fn process_pending_deliveries(&self) -> Result<(), DomainError>;
}

pub struct WebhookDispatcherImpl<R: WebhookRepository> {
    webhook_repository: Arc<R>,
    http_client: Client,
}

impl<R: WebhookRepository> WebhookDispatcherImpl<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("The-Warehouse-Hub-Webhook-Dispatcher/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            webhook_repository,
            http_client,
        }
    }

    /// Send a webhook to a specific URL
    async fn send_webhook(
        &self,
        webhook: &Webhook,
        event: &WebhookEvent,
        delivery: &WebhookDelivery,
    ) -> Result<(bool, Option<i32>, Option<String>, Option<String>), DomainError> {
        // Create the webhook payload
        let payload = serde_json::json!({
            "id": event.id,
            "event_type": event.event_type.as_str(),
            "timestamp": event.created_at.to_rfc3339(),
            "data": event.payload
        });

        // Create HMAC signature for verification
        let signature = self.create_signature(&webhook.secret, &payload)?;

        // Prepare the request
        let request = self
            .http_client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "The-Warehouse-Hub-Webhook-Dispatcher/1.0")
            .header("X-Webhook-ID", webhook.id.to_string())
            .header("X-Webhook-Event", event.event_type.as_str())
            .header("X-Webhook-Delivery", delivery.id.to_string())
            .header("X-Webhook-Signature", signature)
            .json(&payload);

        // Send the request
        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let status_code = status.as_u16() as i32;

                // Read response body
                let response_body = match response.text().await {
                    Ok(text) => Some(text),
                    Err(_) => None,
                };

                // Consider 2xx status codes as success
                let success = status.is_success();

                Ok((success, Some(status_code), response_body, None))
            }
            Err(e) => {
                // Handle network errors, timeouts, etc.
                let error_message = if e.is_timeout() {
                    "Request timed out".to_string()
                } else if e.is_connect() {
                    "Connection failed".to_string()
                } else {
                    format!("HTTP request failed: {}", e)
                };

                Ok((false, None, None, Some(error_message)))
            }
        }
    }

    /// Create HMAC signature for webhook verification
    fn create_signature(
        &self,
        secret: &str,
        payload: &serde_json::Value,
    ) -> Result<String, DomainError> {
        use hmac::digest::KeyInit;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let payload_str = serde_json::to_string(payload).map_err(|e| {
            DomainError::ValidationError(format!("Failed to serialize payload: {}", e))
        })?;

        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(secret.as_bytes())
            .map_err(|e| DomainError::ValidationError(format!("Failed to create HMAC: {}", e)))?;

        mac.update(payload_str.as_bytes());

        let result = mac.finalize();
        let signature = format!("sha256={}", hex::encode(result.into_bytes()));

        Ok(signature)
    }
}

#[async_trait]
impl<R: WebhookRepository> WebhookDispatcher for WebhookDispatcherImpl<R> {
    async fn dispatch_event(&self, event: &WebhookEvent) -> Result<(), DomainError> {
        // Find all webhooks subscribed to this event type
        let webhooks = self
            .webhook_repository
            .get_webhooks_for_event(&event.event_type)
            .await?;

        if webhooks.is_empty() {
            return Ok(()); // No webhooks to dispatch to
        }

        // Create deliveries for each webhook
        for webhook in &webhooks {
            let delivery = WebhookDelivery::new(webhook.id, event.id);

            // Store the delivery in the database
            self.webhook_repository.create_delivery(&delivery).await?;

            // Immediately attempt to send the webhook
            self.retry_delivery(delivery.id).await?;
        }

        Ok(())
    }

    async fn retry_delivery(&self, delivery_id: Uuid) -> Result<(), DomainError> {
        // Get the delivery
        let mut delivery = match self.webhook_repository.get_delivery(delivery_id).await? {
            Some(delivery) => delivery,
            None => {
                return Err(DomainError::ValidationError(
                    "Delivery not found".to_string(),
                ))
            }
        };

        // Check if delivery should be retried
        if !delivery.should_retry() {
            return Ok(()); // Nothing to do
        }

        // Get the webhook and event
        let webhook = match self
            .webhook_repository
            .get_webhook(delivery.webhook_id)
            .await?
        {
            Some(webhook) => webhook,
            None => {
                // Webhook was deleted, mark delivery as failed
                delivery.record_attempt(false, None, None, Some("Webhook not found".to_string()));
                self.webhook_repository.update_delivery(&delivery).await?;
                return Ok(());
            }
        };

        let event = match self
            .webhook_repository
            .get_recent_events(1, 0)
            .await?
            .into_iter()
            .find(|e| e.id == delivery.event_id)
        {
            Some(event) => event,
            None => {
                // Event not found, mark delivery as failed
                delivery.record_attempt(false, None, None, Some("Event not found".to_string()));
                self.webhook_repository.update_delivery(&delivery).await?;
                return Ok(());
            }
        };

        // Send the webhook
        let (success, response_status, response_body, error_message) =
            self.send_webhook(&webhook, &event, &delivery).await?;

        // Record the attempt
        delivery.record_attempt(success, response_status, response_body, error_message);

        // Update the delivery in the database
        self.webhook_repository.update_delivery(&delivery).await?;

        // Update webhook statistics
        let mut updated_webhook = webhook;
        updated_webhook.record_delivery_attempt(success);
        self.webhook_repository
            .update_webhook(&updated_webhook)
            .await?;

        Ok(())
    }

    async fn process_pending_deliveries(&self) -> Result<(), DomainError> {
        // Get pending deliveries (limit to avoid processing too many at once)
        let pending_deliveries = self.webhook_repository.get_pending_deliveries(50).await?;

        for delivery in pending_deliveries {
            // Retry each delivery
            if let Err(e) = self.retry_delivery(delivery.id).await {
                // Log error but continue processing other deliveries
                eprintln!("Failed to retry delivery {}: {}", delivery.id, e);
            }
        }

        Ok(())
    }
}
