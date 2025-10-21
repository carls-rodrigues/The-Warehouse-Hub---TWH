use crate::domain::entities::webhook::{Webhook, WebhookEventType, WebhookStatus};
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterWebhookRequest {
    pub url: String,
    pub secret: String,
    pub events: Vec<WebhookEventType>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterWebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<WebhookEventType>,
    pub name: Option<String>,
    pub status: WebhookStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct RegisterWebhookUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> RegisterWebhookUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(
        &self,
        request: RegisterWebhookRequest,
        user_id: Uuid,
    ) -> Result<RegisterWebhookResponse, DomainError> {
        // Validate URL format
        if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
            return Err(DomainError::ValidationError(
                "Webhook URL must start with http:// or https://".to_string(),
            ));
        }

        // Validate secret length (should be at least 32 characters for security)
        if request.secret.len() < 32 {
            return Err(DomainError::ValidationError(
                "Webhook secret must be at least 32 characters long".to_string(),
            ));
        }

        // Validate events list is not empty
        if request.events.is_empty() {
            return Err(DomainError::ValidationError(
                "At least one event type must be specified".to_string(),
            ));
        }

        // Create webhook entity
        let webhook = Webhook::new(request.url, request.secret, request.events, user_id)?;

        // Save to repository
        self.webhook_repository.create_webhook(&webhook).await?;

        Ok(RegisterWebhookResponse {
            id: webhook.id,
            url: webhook.url,
            events: webhook.events,
            name: None, // Webhook entity doesn't have name field
            status: webhook.status,
            created_at: webhook.created_at,
            updated_at: webhook.updated_at,
        })
    }
}
