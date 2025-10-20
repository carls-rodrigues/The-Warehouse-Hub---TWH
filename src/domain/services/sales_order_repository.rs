use crate::domain::entities::sales_order::{
    SalesOrder, SalesOrderLine, ShipLineRequest, StockMovement,
};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SalesOrderRepository: Send + Sync {
    async fn create(&self, sales_order: &SalesOrder) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<(SalesOrder, Vec<SalesOrderLine>)>, DomainError>;
    async fn find_by_so_number(
        &self,
        so_number: &str,
    ) -> Result<Option<(SalesOrder, Vec<SalesOrderLine>)>, DomainError>;
    async fn update(&self, sales_order: &SalesOrder) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(SalesOrder, Vec<SalesOrderLine>)>, DomainError>;
    async fn ship_sales_order(
        &self,
        id: Uuid,
        shipped_lines: Vec<ShipLineRequest>,
        created_by: Uuid,
    ) -> Result<(SalesOrder, Vec<SalesOrderLine>, Vec<StockMovement>), DomainError>;
    async fn reserve_inventory(
        &self,
        id: Uuid,
        created_by: Uuid,
    ) -> Result<Vec<StockMovement>, DomainError>;
}
