use crate::domain::entities::purchase_order::{
    CreatePurchaseOrderRequest, PurchaseOrder, ReceivePurchaseOrderRequest,
};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait PurchaseOrderRepository: Send + Sync {
    /// Find a purchase order by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PurchaseOrder>, DomainError>;

    /// Find a purchase order by its PO number
    async fn find_by_po_number(
        &self,
        po_number: &str,
    ) -> Result<Option<PurchaseOrder>, DomainError>;

    /// Save a new purchase order
    async fn save(&self, po: &PurchaseOrder) -> Result<(), DomainError>;

    /// Update an existing purchase order
    async fn update(&self, po: &PurchaseOrder) -> Result<(), DomainError>;

    /// Delete a purchase order by ID
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// List purchase orders with pagination and optional filtering
    async fn list(
        &self,
        limit: i64,
        offset: i64,
        status_filter: Option<String>,
    ) -> Result<Vec<PurchaseOrder>, DomainError>;

    /// Count total purchase orders
    async fn count(&self, status_filter: Option<String>) -> Result<i64, DomainError>;

    /// Receive items for a purchase order (update lines and create stock movements)
    async fn receive_purchase_order(
        &self,
        po_id: Uuid,
        request: &ReceivePurchaseOrderRequest,
        user_id: Uuid,
    ) -> Result<Vec<crate::domain::entities::inventory::StockMovement>, DomainError>;
}
