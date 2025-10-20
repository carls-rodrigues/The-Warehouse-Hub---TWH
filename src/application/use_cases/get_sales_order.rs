use crate::domain::entities::sales_order::{SalesOrder, SalesOrderLine};
use crate::domain::services::sales_order_repository::SalesOrderRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SalesOrderWithLines {
    pub sales_order: SalesOrder,
    pub lines: Vec<SalesOrderLine>,
}

pub struct GetSalesOrderUseCase<T: SalesOrderRepository> {
    sales_order_repo: T,
}

impl<T: SalesOrderRepository> GetSalesOrderUseCase<T> {
    pub fn new(sales_order_repo: T) -> Self {
        Self { sales_order_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<SalesOrderWithLines, DomainError> {
        let (sales_order, lines) = self
            .sales_order_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Sales order {} not found", id)))?;

        Ok(SalesOrderWithLines { sales_order, lines })
    }

    pub async fn execute_by_number(
        &self,
        so_number: &str,
    ) -> Result<SalesOrderWithLines, DomainError> {
        let (sales_order, lines) = self
            .sales_order_repo
            .find_by_so_number(so_number)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Sales order {} not found", so_number)))?;

        Ok(SalesOrderWithLines { sales_order, lines })
    }
}
