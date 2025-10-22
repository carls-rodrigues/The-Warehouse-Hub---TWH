use crate::domain::entities::sales_order::{SalesOrder, SalesOrderLine};
use crate::domain::entities::webhook::{WebhookEvent, WebhookEventType};
use crate::domain::services::sales_order_repository::SalesOrderRepository;
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
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

pub struct CreateSalesOrderUseCase<T: SalesOrderRepository, D: WebhookDispatcher + 'static> {
    sales_order_repo: Arc<T>,
    webhook_dispatcher: Arc<D>,
}

impl<T: SalesOrderRepository, D: WebhookDispatcher + 'static> CreateSalesOrderUseCase<T, D> {
    pub fn new(sales_order_repo: Arc<T>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            sales_order_repo,
            webhook_dispatcher,
        }
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

        // Dispatch webhook event (non-blocking)
        let webhook_event = WebhookEvent::new(
            WebhookEventType::SalesOrderCreated,
            json!({
                "sales_order": {
                    "id": sales_order.id,
                    "so_number": sales_order.so_number,
                    "customer_id": sales_order.customer_id,
                    "status": match sales_order.status {
                        crate::domain::entities::sales_order::SalesOrderStatus::Draft => "DRAFT",
                        crate::domain::entities::sales_order::SalesOrderStatus::Confirmed => "CONFIRMED",
                        crate::domain::entities::sales_order::SalesOrderStatus::Picking => "PICKING",
                        crate::domain::entities::sales_order::SalesOrderStatus::Shipped => "SHIPPED",
                        crate::domain::entities::sales_order::SalesOrderStatus::Invoiced => "INVOICED",
                        crate::domain::entities::sales_order::SalesOrderStatus::Cancelled => "CANCELLED",
                        crate::domain::entities::sales_order::SalesOrderStatus::Returned => "RETURNED",
                    },
                    "total_amount": sales_order.total_amount,
                    "fulfillment_location_id": sales_order.fulfillment_location_id,
                    "created_at": sales_order.created_at,
                    "lines": sales_order.lines.iter().map(|line| json!({
                        "id": line.id,
                        "item_id": line.item_id,
                        "qty": line.qty,
                        "unit_price": line.unit_price,
                        "tax": line.tax,
                        "reserved": line.reserved
                    })).collect::<Vec<_>>()
                }
            }),
        );

        // Spawn a task to dispatch the webhook asynchronously
        let dispatcher = Arc::clone(&self.webhook_dispatcher);
        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&webhook_event).await {
                eprintln!("Failed to dispatch sales order created webhook: {:?}", e);
            }
        });

        Ok(CreateSalesOrderResponse {
            sales_order,
            stock_movements,
        })
    }
}
