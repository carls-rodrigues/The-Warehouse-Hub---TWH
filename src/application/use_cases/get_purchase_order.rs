use crate::domain::entities::purchase_order::PurchaseOrder;
use crate::domain::services::purchase_order_repository::PurchaseOrderRepository;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPurchaseOrderResponse {
    pub id: Uuid,
    pub po_number: String,
    pub supplier_id: Uuid,
    pub status: String,
    pub expected_date: Option<chrono::DateTime<chrono::Utc>>,
    pub total_amount: f64,
    pub lines: Vec<PurchaseOrderLineResponse>,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

pub struct GetPurchaseOrderUseCase<R: PurchaseOrderRepository> {
    purchase_order_repository: Arc<R>,
}

impl<R: PurchaseOrderRepository> GetPurchaseOrderUseCase<R> {
    pub fn new(purchase_order_repository: Arc<R>) -> Self {
        Self {
            purchase_order_repository,
        }
    }

    pub async fn execute(&self, id: Uuid) -> Result<GetPurchaseOrderResponse, DomainError> {
        let po = self
            .purchase_order_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::ValidationError("Purchase order not found".to_string()))?;

        Ok(GetPurchaseOrderResponse {
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
            expected_date: po.expected_date,
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
            created_by: po.created_by,
            created_at: po.created_at,
            updated_at: po.updated_at,
        })
    }
}
