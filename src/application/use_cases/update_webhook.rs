use crate::domain::entities::webhook::{Webhook, WebhookEventType, WebhookStatus};
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWebhookRequest {
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<Vec<WebhookEventType>>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<WebhookEventType>,
    pub name: Option<String>,
    pub status: WebhookStatus,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct UpdateWebhookUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> UpdateWebhookUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(
        &self,
        webhook_id: Uuid,
        request: UpdateWebhookRequest,
        user_id: Uuid,
    ) -> Result<UpdateWebhookResponse, DomainError> {
        // Get existing webhook
        let webhook_option = self.webhook_repository.get_webhook(webhook_id).await?;
        let mut webhook = webhook_option.ok_or_else(|| {
            DomainError::NotFound(format!("Webhook with id {} not found", webhook_id))
        })?;

        // Verify ownership
        if webhook.created_by != user_id {
            return Err(DomainError::BusinessLogicError(
                "You can only update your own webhooks".to_string(),
            ));
        }

        // Validate URL if provided
        if let Some(ref url) = request.url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(DomainError::ValidationError(
                    "Webhook URL must start with http:// or https://".to_string(),
                ));
            }
            webhook.url = url.clone();
        }

        // Validate secret if provided
        if let Some(ref secret) = request.secret {
            if secret.len() < 32 {
                return Err(DomainError::ValidationError(
                    "Webhook secret must be at least 32 characters long".to_string(),
                ));
            }
            webhook.secret = secret.clone();
        }

        // Validate events if provided
        if let Some(ref events) = request.events {
            if events.is_empty() {
                return Err(DomainError::ValidationError(
                    "At least one event type must be specified".to_string(),
                ));
            }
            webhook.events = events.clone();
        }

        // Update optional fields (none currently supported by Webhook entity)

        // Update status if active flag provided
        if let Some(active) = request.active {
            webhook.update_status(if active {
                WebhookStatus::Active
            } else {
                WebhookStatus::Inactive
            });
        }

        // Update timestamp
        webhook.updated_at = chrono::Utc::now();

        // Save to repository
        self.webhook_repository.update_webhook(&webhook).await?;

        Ok(UpdateWebhookResponse {
            id: webhook.id,
            url: webhook.url,
            events: webhook.events,
            name: None, // Webhook entity doesn't have name field
            status: webhook.status,
            updated_at: webhook.updated_at,
        })
    }
}
