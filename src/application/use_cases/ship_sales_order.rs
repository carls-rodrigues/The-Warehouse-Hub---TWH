use crate::domain::entities::sales_order::{
    SalesOrder, SalesOrderLine, ShipLineRequest, StockMovement,
};
use crate::domain::services::sales_order_repository::SalesOrderRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ShipSalesOrderRequest {
    pub ship_date: Option<DateTime<Utc>>,
    pub lines: Vec<ShipSalesOrderLineRequest>,
    pub tracking: Option<String>,
    pub carrier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShipSalesOrderLineRequest {
    pub so_line_id: Uuid,
    pub qty_shipped: i32,
}

#[derive(Debug, Serialize)]
pub struct ShipSalesOrderResponse {
    pub sales_order: SalesOrder,
    pub stock_movements: Vec<StockMovement>,
}

pub struct ShipSalesOrderUseCase<T: SalesOrderRepository> {
    sales_order_repo: T,
}

impl<T: SalesOrderRepository> ShipSalesOrderUseCase<T> {
    pub fn new(sales_order_repo: T) -> Self {
        Self { sales_order_repo }
    }

    pub async fn execute(
        &self,
        so_id: Uuid,
        request: ShipSalesOrderRequest,
        created_by: Uuid,
    ) -> Result<ShipSalesOrderResponse, DomainError> {
        // Convert request lines to domain objects
        let shipped_lines: Vec<ShipLineRequest> = request
            .lines
            .into_iter()
            .map(|line| ShipLineRequest {
                so_line_id: line.so_line_id,
                qty_shipped: line.qty_shipped,
            })
            .collect();

        // Ship the sales order through the repository
        let (sales_order, lines, stock_movements) = self
            .sales_order_repo
            .ship_sales_order(so_id, shipped_lines, created_by)
            .await?;

        Ok(ShipSalesOrderResponse {
            sales_order,
            stock_movements,
        })
    }
}
