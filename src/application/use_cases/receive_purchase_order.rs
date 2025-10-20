use crate::domain::entities::inventory::StockMovement;
use crate::domain::entities::purchase_order::{
    PurchaseOrder, ReceiveLine, ReceivePurchaseOrderRequest,
};
use crate::domain::services::purchase_order_repository::PurchaseOrderRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivePurchaseOrderUseCaseRequest {
    pub po_id: Uuid,
    pub received_lines: Vec<ReceiveLine>,
    pub receive_date: Option<chrono::DateTime<chrono::Utc>>,
    pub destination_location_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivePurchaseOrderResponse {
    pub po: PurchaseOrderResponse,
    pub stock_movements: Vec<StockMovementResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderResponse {
    pub id: Uuid,
    pub po_number: String,
    pub supplier_id: Uuid,
    pub status: String,
    pub total_amount: f64,
    pub lines: Vec<PurchaseOrderLineResponse>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct StockMovementResponse {
    pub id: Uuid,
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub quantity: i32,
    pub movement_type: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub reason: Option<String>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct ReceivePurchaseOrderUseCase<R: PurchaseOrderRepository> {
    purchase_order_repository: Arc<R>,
}

impl<R: PurchaseOrderRepository> ReceivePurchaseOrderUseCase<R> {
    pub fn new(purchase_order_repository: Arc<R>) -> Self {
        Self {
            purchase_order_repository,
        }
    }

    pub async fn execute(
        &self,
        request: ReceivePurchaseOrderUseCaseRequest,
        user_id: Uuid,
    ) -> Result<ReceivePurchaseOrderResponse, DomainError> {
        // Create the receive request
        let receive_request = ReceivePurchaseOrderRequest {
            received_lines: request.received_lines,
            receive_date: request.receive_date,
            destination_location_id: request.destination_location_id,
        };

        // Receive the purchase order
        let movements = self
            .purchase_order_repository
            .receive_purchase_order(request.po_id, &receive_request, user_id)
            .await?;

        // Get updated PO
        let po = self
            .purchase_order_repository
            .find_by_id(request.po_id)
            .await?
            .ok_or_else(|| {
                DomainError::ValidationError("Purchase order not found after receive".to_string())
            })?;

        Ok(ReceivePurchaseOrderResponse {
            po: PurchaseOrderResponse {
                id: po.id,
                po_number: po.po_number,
                supplier_id: po.supplier_id,
                status: match po.status {
                    crate::domain::entities::purchase_order::PurchaseOrderStatus::Draft => "DRAFT".to_string(),
                    crate::domain::entities::purchase_order::PurchaseOrderStatus::Open => "OPEN".to_string(),
                    crate::domain::entities::purchase_order::PurchaseOrderStatus::Receiving => "RECEIVING".to_string(),
                    crate::domain::entities::purchase_order::PurchaseOrderStatus::PartialReceived => "PARTIAL_RECEIVED".to_string(),
                    crate::domain::entities::purchase_order::PurchaseOrderStatus::Received => "RECEIVED".to_string(),
                    crate::domain::entities::purchase_order::PurchaseOrderStatus::Cancelled => "CANCELLED".to_string(),
                },
                total_amount: po.total_amount,
                lines: po.lines.into_iter().map(|line| PurchaseOrderLineResponse {
                    id: line.id,
                    item_id: line.item_id,
                    qty_ordered: line.qty_ordered,
                    qty_received: line.qty_received,
                    unit_cost: line.unit_cost,
                    line_total: line.line_total,
                }).collect(),
                updated_at: po.updated_at,
            },
            stock_movements: movements.into_iter().map(|movement| StockMovementResponse {
                id: movement.id,
                item_id: movement.item_id,
                location_id: movement.location_id,
                quantity: movement.quantity,
                movement_type: match movement.movement_type {
                    crate::domain::entities::inventory::MovementType::Inbound => "INBOUND".to_string(),
                    crate::domain::entities::inventory::MovementType::Outbound => "OUTBOUND".to_string(),
                    crate::domain::entities::inventory::MovementType::Adjustment => "ADJUSTMENT".to_string(),
                    crate::domain::entities::inventory::MovementType::Transfer => "TRANSFER".to_string(),
                    crate::domain::entities::inventory::MovementType::Initial => "INITIAL".to_string(),
                },
                reference_type: Some(movement.reference_type.as_str().to_string()),
                reference_id: movement.reference_id,
                reason: movement.reason,
                created_by: movement.created_by.unwrap_or_else(|| Uuid::nil()),
                created_at: movement.created_at,
            }).collect(),
        })
    }
}
