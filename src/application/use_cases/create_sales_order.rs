use crate::domain::entities::sales_order::{SalesOrder, SalesOrderLine};
use crate::domain::services::sales_order_repository::SalesOrderRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateSalesOrderRequest {
    pub customer_id: Option<Uuid>,
    pub lines: Vec<CreateSalesOrderLineRequest>,
    pub should_reserve: Option<bool>,
    pub fulfillment_location_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalesOrderLineRequest {
    pub item_id: Uuid,
    pub qty: i32,
    pub unit_price: f64,
}

#[derive(Debug, Serialize)]
pub struct CreateSalesOrderResponse {
    pub sales_order: SalesOrder,
    pub stock_movements: Option<Vec<crate::domain::entities::sales_order::StockMovement>>,
}

pub struct CreateSalesOrderUseCase<T: SalesOrderRepository> {
    sales_order_repo: T,
}

impl<T: SalesOrderRepository> CreateSalesOrderUseCase<T> {
    pub fn new(sales_order_repo: T) -> Self {
        Self { sales_order_repo }
    }

    pub async fn execute(
        &self,
        request: CreateSalesOrderRequest,
        created_by: Uuid,
    ) -> Result<CreateSalesOrderResponse, DomainError> {
        // Validate request
        if request.lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Sales order must have at least one line".to_string(),
            ));
        }

        // Generate SO number (in a real app, this might come from a sequence)
        let so_number = format!("SO-{}", Uuid::new_v4().simple());

        // Create sales order
        let mut sales_order = SalesOrder::new(
            so_number,
            request.customer_id,
            request.fulfillment_location_id,
            created_by,
        )?;

        // Add lines
        for line_req in request.lines {
            let line = SalesOrderLine::new(line_req.item_id, line_req.qty, line_req.unit_price)?;
            sales_order.add_line(line)?;
        }

        // Confirm the order (moves from Draft to Confirmed)
        sales_order.confirm()?;

        // Create in repository
        self.sales_order_repo.create(&sales_order).await?;

        // Handle reservation if requested
        let stock_movements = if request.should_reserve.unwrap_or(true) {
            Some(
                self.sales_order_repo
                    .reserve_inventory(sales_order.id, created_by)
                    .await?,
            )
        } else {
            None
        };

        Ok(CreateSalesOrderResponse {
            sales_order,
            stock_movements,
        })
    }
}
