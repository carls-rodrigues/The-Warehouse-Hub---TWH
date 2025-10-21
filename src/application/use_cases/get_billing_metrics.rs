use crate::domain::services::webhook_repository::WebhookRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct GetBillingMetricsUseCase<R: WebhookRepository> {
    webhook_repository: Arc<R>,
}

impl<R: WebhookRepository> GetBillingMetricsUseCase<R> {
    pub fn new(webhook_repository: Arc<R>) -> Self {
        Self { webhook_repository }
    }

    pub async fn execute(&self) -> Result<BillingMetricsResponse, DomainError> {
        // For now, return mock data since we don't have actual metering infrastructure
        // In a real implementation, this would aggregate data from various sources

        let total_webhook_deliveries =
            sqlx::query!("SELECT COUNT(*) as count FROM webhook_deliveries")
                .fetch_one(&*self.webhook_repository.get_pool())
                .await
                .map_err(|_| DomainError::DatabaseError("Failed to count deliveries".to_string()))?
                .count
                .unwrap_or(0);

        let successful_deliveries = sqlx::query!(
            "SELECT COUNT(*) as count FROM webhook_deliveries WHERE status = 'SUCCESS'"
        )
        .fetch_one(&*self.webhook_repository.get_pool())
        .await
        .map_err(|_| {
            DomainError::DatabaseError("Failed to count successful deliveries".to_string())
        })?
        .count
        .unwrap_or(0);

        let failed_deliveries = sqlx::query!(
            "SELECT COUNT(*) as count FROM webhook_deliveries WHERE status IN ('FAILED', 'TIMEOUT', 'DLQ')"
        )
        .fetch_one(&*self.webhook_repository.get_pool())
        .await
        .map_err(|_| DomainError::DatabaseError("Failed to count failed deliveries".to_string()))?
        .count
        .unwrap_or(0);

        // Mock data for other metrics (would be collected from actual usage)
        Ok(BillingMetricsResponse {
            total_api_calls: 1250,
            storage_used_gb: 2.5,
            active_tenants: 3,
            total_items: 150,
            total_locations: 25,
            total_orders: 89,
            total_transfers: 34,
            webhook_deliveries: WebhookMetrics {
                total: total_webhook_deliveries,
                successful: successful_deliveries,
                failed: failed_deliveries,
            },
            billing_period: BillingPeriod {
                start_date: chrono::Utc::now() - chrono::Duration::days(30),
                end_date: chrono::Utc::now(),
                days_remaining: 0,
            },
        })
    }
}

#[derive(Debug, Serialize)]
pub struct BillingMetricsResponse {
    pub total_api_calls: i64,
    pub storage_used_gb: f64,
    pub active_tenants: i64,
    pub total_items: i64,
    pub total_locations: i64,
    pub total_orders: i64,
    pub total_transfers: i64,
    pub webhook_deliveries: WebhookMetrics,
    pub billing_period: BillingPeriod,
}

#[derive(Debug, Serialize)]
pub struct WebhookMetrics {
    pub total: i64,
    pub successful: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize)]
pub struct BillingPeriod {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub days_remaining: i64,
}
