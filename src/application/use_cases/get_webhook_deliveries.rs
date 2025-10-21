use crate::domain::entities::webhook::{Webhook, WebhookDelivery};
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetWebhookDeliveriesUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> GetWebhookDeliveriesUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(
        &self,
        webhook_id: Uuid,
        user_id: Uuid,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<GetWebhookDeliveriesResponse, DomainError> {
        // Verify webhook ownership
        let webhook = self
            .webhook_repository
            .get_webhook(webhook_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Webhook {} not found", webhook_id)))?;

        if webhook.created_by != user_id {
            return Err(DomainError::BusinessLogicError(
                "You can only view deliveries for your own webhooks".to_string(),
            ));
        }

        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(50).clamp(1, 100);
        let offset = (page - 1) * limit;

        let deliveries = self
            .webhook_repository
            .get_webhook_deliveries(webhook_id, limit, offset)
            .await?;
        let total_count = self
            .webhook_repository
            .count_webhook_deliveries(webhook_id)
            .await?;
        let total_pages = (total_count + limit - 1) / limit;

        Ok(GetWebhookDeliveriesResponse {
            deliveries,
            pagination: PaginationInfo {
                page,
                limit,
                total_count,
                total_pages,
            },
        })
    }
}

pub struct GetWebhookDeliveryDetailsUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> GetWebhookDeliveryDetailsUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(
        &self,
        delivery_id: Uuid,
        user_id: Uuid,
    ) -> Result<WebhookDeliveryDetails, DomainError> {
        // Get delivery
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
                "You can only view deliveries for your own webhooks".to_string(),
            ));
        }

        // Get associated event
        let event = self
            .webhook_repository
            .get_event(delivery.event_id)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Event {} not found", delivery.event_id))
            })?;

        Ok(WebhookDeliveryDetails {
            delivery,
            event,
            webhook_url: webhook.url,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct GetWebhookDeliveriesResponse {
    pub deliveries: Vec<WebhookDelivery>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub limit: i64,
    pub total_count: i64,
    pub total_pages: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct WebhookDeliveryDetails {
    pub delivery: WebhookDelivery,
    pub event: crate::domain::entities::webhook::WebhookEvent,
    pub webhook_url: String,
}
