use crate::domain::entities::webhook::{Webhook, WebhookDelivery, WebhookEvent, WebhookEventType};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    /// Create a new webhook
    async fn create_webhook(&self, webhook: &Webhook) -> Result<(), DomainError>;

    /// Get webhook by ID
    async fn get_webhook(&self, id: Uuid) -> Result<Option<Webhook>, DomainError>;

    /// Get all webhooks for a user
    async fn get_user_webhooks(&self, user_id: Uuid) -> Result<Vec<Webhook>, DomainError>;

    /// Get webhooks subscribed to a specific event type
    async fn get_webhooks_for_event(
        &self,
        event_type: &WebhookEventType,
    ) -> Result<Vec<Webhook>, DomainError>;

    /// Update webhook
    async fn update_webhook(&self, webhook: &Webhook) -> Result<(), DomainError>;

    /// Delete webhook
    async fn delete_webhook(&self, id: Uuid) -> Result<(), DomainError>;

    /// Create a webhook event
    async fn create_event(&self, event: &WebhookEvent) -> Result<(), DomainError>;

    /// Get recent events with pagination
    async fn get_recent_events(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookEvent>, DomainError>;

    /// Create a webhook delivery attempt
    async fn create_delivery(&self, delivery: &WebhookDelivery) -> Result<(), DomainError>;

    /// Update delivery status
    async fn update_delivery(&self, delivery: &WebhookDelivery) -> Result<(), DomainError>;

    /// Get deliveries for a webhook with pagination
    async fn get_webhook_deliveries(
        &self,
        webhook_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDelivery>, DomainError>;

    /// Get pending deliveries that need to be retried
    async fn get_pending_deliveries(&self, limit: i64)
        -> Result<Vec<WebhookDelivery>, DomainError>;

    /// Get deliveries in DLQ (Dead Letter Queue)
    async fn get_dlq_deliveries(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDelivery>, DomainError>;

    /// Get delivery by ID
    async fn get_delivery(&self, id: Uuid) -> Result<Option<WebhookDelivery>, DomainError>;

    /// Get event by ID
    async fn get_event(&self, id: Uuid) -> Result<Option<WebhookEvent>, DomainError>;

    /// Count deliveries for a webhook
    async fn count_webhook_deliveries(&self, webhook_id: Uuid) -> Result<i64, DomainError>;

    /// Count DLQ deliveries
    async fn count_dlq_deliveries(&self) -> Result<i64, DomainError>;

    /// Clean up old events and deliveries (for maintenance)
    async fn cleanup_old_data(&self, days_old: i32) -> Result<(), DomainError>;

    /// Get database pool for direct queries (used by admin use cases)
    fn get_pool(&self) -> &sqlx::PgPool;
}
