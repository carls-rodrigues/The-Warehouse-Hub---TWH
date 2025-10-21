use crate::domain::entities::sales_order::{
    SalesOrder, SalesOrderLine, ShipLineRequest, StockMovement,
};
use crate::domain::entities::webhook::{WebhookEvent, WebhookEventType};
use crate::domain::services::sales_order_repository::SalesOrderRepository;
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
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

pub struct ShipSalesOrderUseCase<T: SalesOrderRepository, D: WebhookDispatcher + 'static> {
    sales_order_repo: Arc<T>,
    webhook_dispatcher: Arc<D>,
}

impl<T: SalesOrderRepository, D: WebhookDispatcher + 'static> ShipSalesOrderUseCase<T, D> {
    pub fn new(sales_order_repo: Arc<T>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            sales_order_repo,
            webhook_dispatcher,
        }
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

        // Dispatch webhook event (non-blocking)
        let webhook_event = WebhookEvent::new(
            WebhookEventType::SalesOrderUpdated,
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
                    "updated_at": sales_order.updated_at,
                    "lines": lines.iter().map(|line| json!({
                        "id": line.id,
                        "item_id": line.item_id,
                        "qty": line.qty,
                        "unit_price": line.unit_price,
                        "tax": line.tax,
                        "line_total": line.line_total()
                    })).collect::<Vec<_>>()
                },
                "stock_movements": stock_movements.iter().map(|movement| json!({
                    "id": movement.id,
                    "item_id": movement.item_id,
                    "location_id": movement.location_id,
                    "quantity": movement.quantity,
                    "movement_type": match movement.movement_type {
                        crate::domain::entities::inventory::MovementType::Inbound => "INBOUND",
                        crate::domain::entities::inventory::MovementType::Outbound => "OUTBOUND",
                        crate::domain::entities::inventory::MovementType::Adjustment => "ADJUSTMENT",
                        crate::domain::entities::inventory::MovementType::Transfer => "TRANSFER",
                        crate::domain::entities::inventory::MovementType::Initial => "INITIAL",
                    },
                    "reference_type": movement.reference_type.as_str(),
                    "reference_id": movement.reference_id,
                    "reason": movement.reason,
                    "created_by": movement.created_by,
                    "created_at": movement.created_at
                })).collect::<Vec<_>>()
            }),
        );

        // Spawn a task to dispatch the webhook asynchronously
        let dispatcher = Arc::clone(&self.webhook_dispatcher);
        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&webhook_event).await {
                eprintln!("Failed to dispatch sales order updated webhook: {:?}", e);
            }
        });

        Ok(ShipSalesOrderResponse {
            sales_order,
            stock_movements,
        })
    }
}
