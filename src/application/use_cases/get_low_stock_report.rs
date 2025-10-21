use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::{inventory::StockLevel, item::Item},
    services::{
        item_repository::ItemRepository,
        report_service::{LowStockReportItem, ReportService},
        stock_repository::StockRepository,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLowStockReportRequest {
    pub threshold: i32,
    pub limit: i64,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLowStockReportResponse {
    pub items: Vec<LowStockReportItem>,
    pub next_cursor: Option<String>,
}

pub struct GetLowStockReportUseCase<T: ItemRepository, S: StockRepository, R: ReportService> {
    item_repository: Arc<T>,
    stock_repository: Arc<S>,
    report_service: Arc<R>,
}

impl<T: ItemRepository, S: StockRepository, R: ReportService> GetLowStockReportUseCase<T, S, R> {
    pub fn new(item_repository: Arc<T>, stock_repository: Arc<S>, report_service: Arc<R>) -> Self {
        Self {
            item_repository,
            stock_repository,
            report_service,
        }
    }

    pub async fn execute(
        &self,
        request: GetLowStockReportRequest,
    ) -> Result<GetLowStockReportResponse, String> {
        let response = self
            .report_service
            .generate_low_stock_report(request.threshold, request.limit, request.cursor)
            .await
            .map_err(|e| format!("Failed to generate low stock report: {}", e))?;

        Ok(GetLowStockReportResponse {
            items: response.items,
            next_cursor: response.next_cursor,
        })
    }
}
