use crate::domain::entities::webhook::{
    DeliveryStatus, Webhook, WebhookDelivery, WebhookEvent, WebhookEventType, WebhookStatus,
};
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresWebhookRepository {
    pool: Arc<PgPool>,
}

impl PostgresWebhookRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebhookRepository for PostgresWebhookRepository {
    async fn create_webhook(&self, webhook: &Webhook) -> Result<(), DomainError> {
        let events: Vec<String> = webhook
            .events
            .iter()
            .map(|e| e.as_str().to_string())
            .collect();

        sqlx::query!(
            r#"
            INSERT INTO webhooks (
                id, url, secret, events, status, created_by,
                created_at, updated_at, last_delivery_at, failure_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            webhook.id,
            webhook.url,
            webhook.secret,
            &events,
            webhook.status.as_str(),
            webhook.created_by,
            webhook.created_at,
            webhook.updated_at,
            webhook.last_delivery_at,
            webhook.failure_count
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to create webhook: {}", e)))?;

        Ok(())
    }

    async fn get_webhook(&self, id: Uuid) -> Result<Option<Webhook>, DomainError> {
        let row = sqlx::query!(
            r#"
            SELECT id, url, secret, events, status, created_by,
                   created_at, updated_at, last_delivery_at, failure_count
            FROM webhooks
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to get webhook: {}", e)))?;

        match row {
            Some(row) => {
                let events: Vec<WebhookEventType> = row
                    .events
                    .iter()
                    .filter_map(|e| WebhookEventType::from_str(e).ok())
                    .collect();

                let status = WebhookStatus::from_str(&row.status).map_err(|e| {
                    DomainError::DatabaseError(format!("Invalid webhook status: {}", e))
                })?;

                Ok(Some(Webhook {
                    id: row.id,
                    url: row.url,
                    secret: row.secret,
                    events,
                    status,
                    created_by: row.created_by,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    last_delivery_at: row.last_delivery_at,
                    failure_count: row.failure_count,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_webhooks(&self, user_id: Uuid) -> Result<Vec<Webhook>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, url, secret, events, status, created_by,
                   created_at, updated_at, last_delivery_at, failure_count
            FROM webhooks
            WHERE created_by = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to get user webhooks: {}", e)))?;

        let mut webhooks = Vec::new();
        for row in rows {
            let events: Vec<WebhookEventType> = row
                .events
                .iter()
                .filter_map(|e| WebhookEventType::from_str(e).ok())
                .collect();

            let status = WebhookStatus::from_str(&row.status).map_err(|e| {
                DomainError::DatabaseError(format!("Invalid webhook status: {}", e))
            })?;

            webhooks.push(Webhook {
                id: row.id,
                url: row.url,
                secret: row.secret,
                events,
                status,
                created_by: row.created_by,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_delivery_at: row.last_delivery_at,
                failure_count: row.failure_count,
            });
        }

        Ok(webhooks)
    }

    async fn get_webhooks_for_event(
        &self,
        event_type: &WebhookEventType,
    ) -> Result<Vec<Webhook>, DomainError> {
        let event_str = event_type.as_str();

        let rows = sqlx::query!(
            r#"
            SELECT id, url, secret, events, status, created_by,
                   created_at, updated_at, last_delivery_at, failure_count
            FROM webhooks
            WHERE status = 'ACTIVE' AND $1 = ANY(events)
            "#,
            event_str
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get webhooks for event: {}", e))
        })?;

        let mut webhooks = Vec::new();
        for row in rows {
            let events: Vec<WebhookEventType> = row
                .events
                .iter()
                .filter_map(|e| WebhookEventType::from_str(e).ok())
                .collect();

            let status = WebhookStatus::from_str(&row.status).map_err(|e| {
                DomainError::DatabaseError(format!("Invalid webhook status: {}", e))
            })?;

            webhooks.push(Webhook {
                id: row.id,
                url: row.url,
                secret: row.secret,
                events,
                status,
                created_by: row.created_by,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_delivery_at: row.last_delivery_at,
                failure_count: row.failure_count,
            });
        }

        Ok(webhooks)
    }

    async fn update_webhook(&self, webhook: &Webhook) -> Result<(), DomainError> {
        let events: Vec<String> = webhook
            .events
            .iter()
            .map(|e| e.as_str().to_string())
            .collect();

        sqlx::query!(
            r#"
            UPDATE webhooks
            SET url = $2, secret = $3, events = $4, status = $5,
                updated_at = $6, last_delivery_at = $7, failure_count = $8
            WHERE id = $1
            "#,
            webhook.id,
            webhook.url,
            webhook.secret,
            &events,
            webhook.status.as_str(),
            webhook.updated_at,
            webhook.last_delivery_at,
            webhook.failure_count
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to update webhook: {}", e)))?;

        Ok(())
    }

    async fn delete_webhook(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM webhooks WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to delete webhook: {}", e)))?;

        Ok(())
    }

    async fn create_event(&self, event: &WebhookEvent) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            INSERT INTO webhook_events (id, event_type, payload, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
            event.id,
            event.event_type.as_str(),
            event.payload,
            event.created_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to create webhook event: {}", e))
        })?;

        Ok(())
    }

    async fn get_recent_events(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookEvent>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, event_type, payload, created_at
            FROM webhook_events
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to get recent events: {}", e)))?;

        let mut events = Vec::new();
        for row in rows {
            let event_type = WebhookEventType::from_str(&row.event_type)
                .map_err(|e| DomainError::DatabaseError(format!("Invalid event type: {}", e)))?;

            events.push(WebhookEvent {
                id: row.id,
                event_type,
                payload: row.payload,
                created_at: row.created_at,
            });
        }

        Ok(events)
    }

    async fn create_delivery(&self, delivery: &WebhookDelivery) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            INSERT INTO webhook_deliveries (
                id, webhook_id, event_id, status, attempt_count,
                last_attempt_at, next_attempt_at, response_status,
                response_body, error_message, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            delivery.id,
            delivery.webhook_id,
            delivery.event_id,
            delivery.status.as_str(),
            delivery.attempt_count,
            delivery.last_attempt_at,
            delivery.next_attempt_at,
            delivery.response_status,
            delivery.response_body,
            delivery.error_message,
            delivery.created_at,
            delivery.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to create webhook delivery: {}", e))
        })?;

        Ok(())
    }

    async fn update_delivery(&self, delivery: &WebhookDelivery) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            UPDATE webhook_deliveries
            SET status = $2, attempt_count = $3, last_attempt_at = $4,
                next_attempt_at = $5, response_status = $6,
                response_body = $7, error_message = $8, updated_at = $9
            WHERE id = $1
            "#,
            delivery.id,
            delivery.status.as_str(),
            delivery.attempt_count,
            delivery.last_attempt_at,
            delivery.next_attempt_at,
            delivery.response_status,
            delivery.response_body,
            delivery.error_message,
            delivery.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to update webhook delivery: {}", e))
        })?;

        Ok(())
    }

    async fn get_webhook_deliveries(
        &self,
        webhook_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDelivery>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, webhook_id, event_id, status, attempt_count,
                   last_attempt_at, next_attempt_at, response_status,
                   response_body, error_message, created_at, updated_at
            FROM webhook_deliveries
            WHERE webhook_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            webhook_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get webhook deliveries: {}", e))
        })?;

        let mut deliveries = Vec::new();
        for row in rows {
            let status = DeliveryStatus::from_str(&row.status).map_err(|e| {
                DomainError::DatabaseError(format!("Invalid delivery status: {}", e))
            })?;

            deliveries.push(WebhookDelivery {
                id: row.id,
                webhook_id: row.webhook_id,
                event_id: row.event_id,
                status,
                attempt_count: row.attempt_count,
                last_attempt_at: row.last_attempt_at,
                next_attempt_at: row.next_attempt_at,
                response_status: row.response_status,
                response_body: row.response_body,
                error_message: row.error_message,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(deliveries)
    }

    async fn get_pending_deliveries(
        &self,
        limit: i64,
    ) -> Result<Vec<WebhookDelivery>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, webhook_id, event_id, status, attempt_count,
                   last_attempt_at, next_attempt_at, response_status,
                   response_body, error_message, created_at, updated_at
            FROM webhook_deliveries
            WHERE status IN ('PENDING', 'FAILED')
              AND next_attempt_at <= NOW()
              AND attempt_count < 5
            ORDER BY next_attempt_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to get pending deliveries: {}", e))
        })?;

        let mut deliveries = Vec::new();
        for row in rows {
            let status = DeliveryStatus::from_str(&row.status).map_err(|e| {
                DomainError::DatabaseError(format!("Invalid delivery status: {}", e))
            })?;

            deliveries.push(WebhookDelivery {
                id: row.id,
                webhook_id: row.webhook_id,
                event_id: row.event_id,
                status,
                attempt_count: row.attempt_count,
                last_attempt_at: row.last_attempt_at,
                next_attempt_at: row.next_attempt_at,
                response_status: row.response_status,
                response_body: row.response_body,
                error_message: row.error_message,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(deliveries)
    }

    async fn get_dlq_deliveries(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WebhookDelivery>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, webhook_id, event_id, status, attempt_count,
                   last_attempt_at, next_attempt_at, response_status,
                   response_body, error_message, created_at, updated_at
            FROM webhook_deliveries
            WHERE status = 'DLQ'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to get DLQ deliveries: {}", e)))?;

        let mut deliveries = Vec::new();
        for row in rows {
            let status = DeliveryStatus::from_str(&row.status).map_err(|e| {
                DomainError::DatabaseError(format!("Invalid delivery status: {}", e))
            })?;

            deliveries.push(WebhookDelivery {
                id: row.id,
                webhook_id: row.webhook_id,
                event_id: row.event_id,
                status,
                attempt_count: row.attempt_count,
                last_attempt_at: row.last_attempt_at,
                next_attempt_at: row.next_attempt_at,
                response_status: row.response_status,
                response_body: row.response_body,
                error_message: row.error_message,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(deliveries)
    }

    async fn get_delivery(&self, id: Uuid) -> Result<Option<WebhookDelivery>, DomainError> {
        let row = sqlx::query!(
            r#"
            SELECT id, webhook_id, event_id, status, attempt_count,
                   last_attempt_at, next_attempt_at, response_status,
                   response_body, error_message, created_at, updated_at
            FROM webhook_deliveries
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to get delivery: {}", e)))?;

        match row {
            Some(row) => {
                let status = DeliveryStatus::from_str(&row.status).map_err(|e| {
                    DomainError::DatabaseError(format!("Invalid delivery status: {}", e))
                })?;

                Ok(Some(WebhookDelivery {
                    id: row.id,
                    webhook_id: row.webhook_id,
                    event_id: row.event_id,
                    status,
                    attempt_count: row.attempt_count,
                    last_attempt_at: row.last_attempt_at,
                    next_attempt_at: row.next_attempt_at,
                    response_status: row.response_status,
                    response_body: row.response_body,
                    error_message: row.error_message,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn cleanup_old_data(&self, days_old: i32) -> Result<(), DomainError> {
        // Clean up old events (keep last 30 days)
        sqlx::query!(
            r#"
            DELETE FROM webhook_events
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            "#,
            days_old as f64
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(format!("Failed to cleanup old events: {}", e)))?;

        // Clean up old successful deliveries (keep last 7 days)
        sqlx::query!(
            r#"
            DELETE FROM webhook_deliveries
            WHERE status = 'SUCCESS' AND created_at < NOW() - INTERVAL '1 day' * $1
            "#,
            days_old as f64
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::DatabaseError(format!("Failed to cleanup old deliveries: {}", e))
        })?;

        Ok(())
    }
}
