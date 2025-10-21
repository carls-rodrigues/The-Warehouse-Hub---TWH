use crate::domain::entities::webhook::WebhookDelivery;
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct ListDlqDeliveriesUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> ListDlqDeliveriesUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(
        &self,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<ListDlqDeliveriesResponse, DomainError> {
        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(50).clamp(1, 100);
        let offset = (page - 1) * limit;

        let deliveries = self
            .webhook_repository
            .get_dlq_deliveries(limit, offset)
            .await?;
        let total_count = self.webhook_repository.count_dlq_deliveries().await?;
        let total_pages = (total_count + limit - 1) / limit;

        Ok(ListDlqDeliveriesResponse {
            deliveries,
            pagination: PaginationMeta {
                page,
                per_page: limit,
                total: total_count,
                total_pages,
            },
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ListDlqDeliveriesResponse {
    pub deliveries: Vec<WebhookDelivery>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}
