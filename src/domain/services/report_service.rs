use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{inventory::StockLevel, item::Item};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockReportItem {
    pub item: Item,
    pub stock: StockLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockValuationReportItem {
    pub item: Item,
    pub valuation: f64,
}

#[async_trait]
pub trait ReportService: Send + Sync {
    async fn generate_low_stock_report(
        &self,
        threshold: i32,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<LowStockReportResponse, String>;

    async fn generate_stock_valuation_report(
        &self,
        location_id: Option<Uuid>,
        valuation_method: String,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<StockValuationResponse, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockReportResponse {
    pub items: Vec<LowStockReportItem>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockValuationResponse {
    pub items: Vec<StockValuationReportItem>,
    pub next_cursor: Option<String>,
}
