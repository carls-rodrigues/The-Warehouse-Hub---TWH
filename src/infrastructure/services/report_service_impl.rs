use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    entities::{inventory::StockLevel, item::Item},
    services::{
        item_repository::ItemRepository,
        report_service::{
            LowStockReportItem, LowStockReportResponse, ReportService, StockValuationReportItem,
            StockValuationResponse,
        },
        stock_repository::StockRepository,
    },
};

pub struct ReportServiceImpl<T: ItemRepository, S: StockRepository> {
    item_repository: Arc<T>,
    stock_repository: Arc<S>,
}

impl<T: ItemRepository, S: StockRepository> ReportServiceImpl<T, S> {
    pub fn new(item_repository: Arc<T>, stock_repository: Arc<S>) -> Self {
        Self {
            item_repository,
            stock_repository,
        }
    }
}

#[async_trait]
impl<T: ItemRepository, S: StockRepository> ReportService for ReportServiceImpl<T, S> {
    async fn generate_low_stock_report(
        &self,
        threshold: i32,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<LowStockReportResponse, String> {
        // Get all stock levels below threshold
        let stock_levels = self
            .stock_repository
            .get_stock_levels_below_threshold(threshold, limit, cursor)
            .await
            .map_err(|e| format!("Failed to get stock levels: {}", e))?;

        // Get corresponding items
        let mut items = Vec::new();
        for stock_level in &stock_levels.items {
            if let Some(item) = self
                .item_repository
                .find_by_id(stock_level.item_id)
                .await
                .map_err(|e| format!("Failed to get item {}: {}", stock_level.item_id, e))?
            {
                items.push(LowStockReportItem {
                    item,
                    stock: stock_level.clone(),
                });
            }
        }

        Ok(LowStockReportResponse {
            items,
            next_cursor: stock_levels.next_cursor,
        })
    }

    async fn generate_stock_valuation_report(
        &self,
        location_id: Option<Uuid>,
        valuation_method: String,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<StockValuationResponse, String> {
        // Get stock levels for the specified location (or all locations if none specified)
        let stock_levels = if let Some(location_id) = location_id {
            self.stock_repository
                .get_stock_levels_by_location(location_id, limit, cursor)
                .await
        } else {
            self.stock_repository
                .get_all_stock_levels(limit, cursor)
                .await
        }
        .map_err(|e| format!("Failed to get stock levels: {}", e))?;

        // Calculate valuations
        let mut items = Vec::new();
        for stock_level in &stock_levels.items {
            if let Some(item) = self
                .item_repository
                .find_by_id(stock_level.item_id)
                .await
                .map_err(|e| format!("Failed to get item {}: {}", stock_level.item_id, e))?
            {
                let valuation = self
                    .calculate_item_valuation(&item, &stock_level, &valuation_method)
                    .await?;

                items.push(StockValuationReportItem { item, valuation });
            }
        }

        Ok(StockValuationResponse {
            items,
            next_cursor: stock_levels.next_cursor,
        })
    }
}

impl<T: ItemRepository, S: StockRepository> ReportServiceImpl<T, S> {
    async fn calculate_item_valuation(
        &self,
        item: &Item,
        stock_level: &StockLevel,
        valuation_method: &str,
    ) -> Result<f64, String> {
        match valuation_method {
            "FIFO" => {
                // FIFO: Use the cost_price from the item
                Ok(item.cost_price * stock_level.quantity_on_hand as f64)
            }
            "LIFO" => {
                // LIFO: For simplicity, use current cost_price (would need movement history for true LIFO)
                Ok(item.cost_price * stock_level.quantity_on_hand as f64)
            }
            "AVG" => {
                // AVG: Use current cost_price (would need to calculate average from movements)
                Ok(item.cost_price * stock_level.quantity_on_hand as f64)
            }
            _ => Err(format!(
                "Unsupported valuation method: {}",
                valuation_method
            )),
        }
    }
}
