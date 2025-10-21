use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteWebhookUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> DeleteWebhookUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(&self, webhook_id: Uuid, user_id: Uuid) -> Result<(), DomainError> {
        // Get existing webhook to verify ownership
        let webhook_option = self.webhook_repository.get_webhook(webhook_id).await?;
        let webhook = webhook_option.ok_or_else(|| {
            DomainError::NotFound(format!("Webhook with id {} not found", webhook_id))
        })?;

        // Verify ownership
        if webhook.created_by != user_id {
            return Err(DomainError::BusinessLogicError(
                "You can only delete your own webhooks".to_string(),
            ));
        }

        // Delete the webhook
        self.webhook_repository.delete_webhook(webhook_id).await?;

        Ok(())
    }
}
