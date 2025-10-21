use crate::domain::entities::purchase_order::{
    CreatePurchaseOrderLine, CreatePurchaseOrderRequest, PurchaseOrder,
};
use crate::domain::entities::webhook::{WebhookEvent, WebhookEventType};
use crate::domain::services::purchase_order_repository::PurchaseOrderRepository;
use crate::domain::services::webhook_dispatcher::WebhookDispatcher;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePurchaseOrderUseCaseRequest {
    pub supplier_id: Uuid,
    pub expected_date: Option<chrono::DateTime<chrono::Utc>>,
    pub lines: Vec<CreatePurchaseOrderLine>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePurchaseOrderResponse {
    pub id: Uuid,
    pub po_number: String,
    pub supplier_id: Uuid,
    pub status: String,
    pub total_amount: f64,
    pub lines: Vec<PurchaseOrderLineResponse>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderLineResponse {
    pub id: Uuid,
    pub item_id: Uuid,
    pub qty_ordered: i32,
    pub qty_received: i32,
    pub unit_cost: f64,
    pub line_total: f64,
}

pub struct CreatePurchaseOrderUseCase<R: PurchaseOrderRepository, D: WebhookDispatcher + 'static> {
    purchase_order_repository: Arc<R>,
    webhook_dispatcher: Arc<D>,
}

impl<R: PurchaseOrderRepository, D: WebhookDispatcher + 'static> CreatePurchaseOrderUseCase<R, D> {
    pub fn new(purchase_order_repository: Arc<R>, webhook_dispatcher: Arc<D>) -> Self {
        Self {
            purchase_order_repository,
            webhook_dispatcher,
        }
    }

    pub async fn execute(
        &self,
        request: CreatePurchaseOrderUseCaseRequest,
        created_by: Uuid,
    ) -> Result<CreatePurchaseOrderResponse, DomainError> {
        // Create the purchase order
        let po = PurchaseOrder::new(
            request.supplier_id,
            request.lines,
            request.expected_date,
            created_by,
        )?;

        // Save to repository
        self.purchase_order_repository.save(&po).await?;

        // Dispatch webhook event (non-blocking)
        let webhook_event = WebhookEvent::new(
            WebhookEventType::PurchaseOrderCreated,
            json!({
                "purchase_order": {
                    "id": po.id,
                    "po_number": po.po_number,
                    "supplier_id": po.supplier_id,
                    "status": match po.status {
                        crate::domain::entities::purchase_order::PurchaseOrderStatus::Draft => "DRAFT",
                        crate::domain::entities::purchase_order::PurchaseOrderStatus::Open => "OPEN",
                        crate::domain::entities::purchase_order::PurchaseOrderStatus::Receiving => "RECEIVING",
                        crate::domain::entities::purchase_order::PurchaseOrderStatus::PartialReceived => "PARTIAL_RECEIVED",
                        crate::domain::entities::purchase_order::PurchaseOrderStatus::Received => "RECEIVED",
                        crate::domain::entities::purchase_order::PurchaseOrderStatus::Cancelled => "CANCELLED",
                    },
                    "total_amount": po.total_amount,
                    "expected_date": po.expected_date,
                    "created_at": po.created_at,
                    "lines": po.lines.iter().map(|line| json!({
                        "id": line.id,
                        "item_id": line.item_id,
                        "qty_ordered": line.qty_ordered,
                        "qty_received": line.qty_received,
                        "unit_cost": line.unit_cost,
                        "line_total": line.line_total
                    })).collect::<Vec<_>>()
                }
            }),
        );

        // Spawn a task to dispatch the webhook asynchronously
        let dispatcher = Arc::clone(&self.webhook_dispatcher);
        tokio::spawn(async move {
            if let Err(e) = dispatcher.dispatch_event(&webhook_event).await {
                eprintln!("Failed to dispatch purchase order created webhook: {:?}", e);
            }
        });

        // Return response
        Ok(CreatePurchaseOrderResponse {
            id: po.id,
            po_number: po.po_number,
            supplier_id: po.supplier_id,
            status: match po.status {
                crate::domain::entities::purchase_order::PurchaseOrderStatus::Draft => {
                    "DRAFT".to_string()
                }
                crate::domain::entities::purchase_order::PurchaseOrderStatus::Open => {
                    "OPEN".to_string()
                }
                crate::domain::entities::purchase_order::PurchaseOrderStatus::Receiving => {
                    "RECEIVING".to_string()
                }
                crate::domain::entities::purchase_order::PurchaseOrderStatus::PartialReceived => {
                    "PARTIAL_RECEIVED".to_string()
                }
                crate::domain::entities::purchase_order::PurchaseOrderStatus::Received => {
                    "RECEIVED".to_string()
                }
                crate::domain::entities::purchase_order::PurchaseOrderStatus::Cancelled => {
                    "CANCELLED".to_string()
                }
            },
            total_amount: po.total_amount,
            lines: po
                .lines
                .into_iter()
                .map(|line| PurchaseOrderLineResponse {
                    id: line.id,
                    item_id: line.item_id,
                    qty_ordered: line.qty_ordered,
                    qty_received: line.qty_received,
                    unit_cost: line.unit_cost,
                    line_total: line.line_total,
                })
                .collect(),
            created_at: po.created_at,
        })
    }
}
