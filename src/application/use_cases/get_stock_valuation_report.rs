use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::services::report_service::{ReportService, StockValuationReportItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockValuationReportRequest {
    pub location_id: Option<Uuid>,
    pub valuation_method: String,
    pub limit: i64,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockValuationReportResponse {
    pub items: Vec<StockValuationReportItem>,
    pub next_cursor: Option<String>,
}

pub struct GetStockValuationReportUseCase<R: ReportService> {
    report_service: Arc<R>,
}

impl<R: ReportService> GetStockValuationReportUseCase<R> {
    pub fn new(report_service: Arc<R>) -> Self {
        Self { report_service }
    }

    pub async fn execute(
        &self,
        request: GetStockValuationReportRequest,
    ) -> Result<GetStockValuationReportResponse, String> {
        let response = self
            .report_service
            .generate_stock_valuation_report(
                request.location_id,
                request.valuation_method,
                request.limit,
                request.cursor,
            )
            .await
            .map_err(|e| format!("Failed to generate stock valuation report: {}", e))?;

        Ok(GetStockValuationReportResponse {
            items: response.items,
            next_cursor: response.next_cursor,
        })
    }
}
