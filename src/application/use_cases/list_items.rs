use crate::domain::services::item_repository::ItemRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemsRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: uuid::Uuid,
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: String,
    pub cost_price: f64,
    pub sale_price: Option<f64>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListItemsResponse {
    pub items: Vec<ItemSummary>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

pub struct ListItemsUseCase<R: ItemRepository> {
    item_repository: Arc<R>,
}

impl<R: ItemRepository> ListItemsUseCase<R> {
    pub fn new(item_repository: Arc<R>) -> Self {
        Self { item_repository }
    }

    pub async fn execute(
        &self,
        request: ListItemsRequest,
    ) -> Result<ListItemsResponse, DomainError> {
        // Set defaults for pagination
        let limit = request.limit.unwrap_or(50).min(1000); // Max 1000 items per page
        let offset = request.offset.unwrap_or(0).max(0); // Ensure non-negative offset

        // Get items and total count in parallel
        let (items, total_count) = tokio::try_join!(
            self.item_repository.list(limit, offset),
            self.item_repository.count()
        )?;

        // Convert to summary format
        let items_summary = items
            .into_iter()
            .map(|item| ItemSummary {
                id: item.id,
                sku: item.sku,
                name: item.name,
                category: item.category,
                unit: item.unit,
                cost_price: item.cost_price,
                sale_price: item.sale_price,
                active: item.active,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
            .collect();

        Ok(ListItemsResponse {
            items: items_summary,
            total_count,
            limit,
            offset,
        })
    }
}
