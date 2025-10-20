use crate::domain::entities::sales_order::{
    SalesOrder, SalesOrderLine, SalesOrderStatus, ShipLineRequest, StockMovement,
};
use crate::domain::services::sales_order_repository::SalesOrderRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use std::sync::Arc;

pub struct PostgresSalesOrderRepository {
    pool: Arc<PgPool>,
}

impl PostgresSalesOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SalesOrderRepository for PostgresSalesOrderRepository {
    async fn create(&self, sales_order: &SalesOrder) -> Result<(), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert sales order
        sqlx::query(
            r#"
            INSERT INTO sales_orders (id, so_number, customer_id, status, total_amount, fulfillment_location_id, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(sales_order.id)
        .bind(&sales_order.so_number)
        .bind(sales_order.customer_id)
        .bind(sales_order.status.as_str())
        .bind(sales_order.total_amount)
        .bind(sales_order.fulfillment_location_id)
        .bind(sales_order.created_by)
        .bind(sales_order.created_at)
        .bind(sales_order.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert sales order lines
        for line in &sales_order.lines {
            sqlx::query(
                r#"
                INSERT INTO sales_order_lines (id, so_id, item_id, qty, unit_price, tax, reserved, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(line.id)
            .bind(line.so_id)
            .bind(line.item_id)
            .bind(line.qty)
            .bind(line.unit_price)
            .bind(line.tax)
            .bind(line.reserved)
            .bind(line.created_at)
            .bind(line.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<(SalesOrder, Vec<SalesOrderLine>)>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                so.id, so.so_number, so.customer_id, so.status, so.total_amount, so.fulfillment_location_id,
                so.created_by, so.created_at, so.updated_at,
                sol.id as line_id, sol.item_id, sol.qty, sol.unit_price, sol.tax, sol.reserved,
                sol.created_at as line_created_at, sol.updated_at as line_updated_at
            FROM sales_orders so
            LEFT JOIN sales_order_lines sol ON so.id = sol.so_id
            WHERE so.id = $1
            ORDER BY sol.created_at
            "#,
        )
        .bind(id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if row.is_empty() {
            return Ok(None);
        }

        let mut sales_order: Option<SalesOrder> = None;
        let mut lines = Vec::new();

        for r in row {
            if sales_order.is_none() {
                let status_str: String = r
                    .try_get("status")
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                let status = SalesOrderStatus::from_str(&status_str)?;

                sales_order = Some(SalesOrder {
                    id: r
                        .try_get("id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    so_number: r
                        .try_get("so_number")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    customer_id: r
                        .try_get("customer_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    status,
                    total_amount: r
                        .try_get("total_amount")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    fulfillment_location_id: r
                        .try_get("fulfillment_location_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    lines: Vec::new(), // Will be set later
                    created_by: r
                        .try_get("created_by")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    updated_at: r
                        .try_get("updated_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                });
            }

            // Add line if it exists
            if let Ok(line_id) = r.try_get::<Uuid, _>("line_id") {
                let line = SalesOrderLine {
                    id: line_id,
                    so_id: r
                        .try_get("id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    item_id: r
                        .try_get("item_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    qty: r
                        .try_get("qty")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    unit_price: r
                        .try_get("unit_price")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    tax: r
                        .try_get("tax")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    reserved: r
                        .try_get("reserved")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    created_at: r
                        .try_get("line_created_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    updated_at: r
                        .try_get("line_updated_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                };
                lines.push(line);
            }
        }

        if let Some(mut so) = sales_order {
            so.lines = lines.clone();
            Ok(Some((so, lines)))
        } else {
            Ok(None)
        }
    }

    async fn find_by_so_number(
        &self,
        so_number: &str,
    ) -> Result<Option<(SalesOrder, Vec<SalesOrderLine>)>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                so.id, so.so_number, so.customer_id, so.status, so.total_amount, so.fulfillment_location_id,
                so.created_by, so.created_at, so.updated_at,
                sol.id as line_id, sol.item_id, sol.qty, sol.unit_price, sol.tax, sol.reserved,
                sol.created_at as line_created_at, sol.updated_at as line_updated_at
            FROM sales_orders so
            LEFT JOIN sales_order_lines sol ON so.id = sol.so_id
            WHERE so.so_number = $1
            ORDER BY sol.created_at
            "#,
        )
        .bind(so_number)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if row.is_empty() {
            return Ok(None);
        }

        let mut sales_order: Option<SalesOrder> = None;
        let mut lines = Vec::new();

        for r in row {
            if sales_order.is_none() {
                let status_str: String = r
                    .try_get("status")
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                let status = SalesOrderStatus::from_str(&status_str)?;

                sales_order = Some(SalesOrder {
                    id: r
                        .try_get("id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    so_number: r
                        .try_get("so_number")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    customer_id: r
                        .try_get("customer_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    status,
                    total_amount: r
                        .try_get("total_amount")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    fulfillment_location_id: r
                        .try_get("fulfillment_location_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    lines: Vec::new(), // Will be set later
                    created_by: r
                        .try_get("created_by")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    updated_at: r
                        .try_get("updated_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                });
            }

            // Add line if it exists
            if let Ok(line_id) = r.try_get::<Uuid, _>("line_id") {
                let line = SalesOrderLine {
                    id: line_id,
                    so_id: r
                        .try_get("id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    item_id: r
                        .try_get("item_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    qty: r
                        .try_get("qty")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    unit_price: r
                        .try_get("unit_price")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    tax: r
                        .try_get("tax")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    reserved: r
                        .try_get("reserved")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    created_at: r
                        .try_get("line_created_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    updated_at: r
                        .try_get("line_updated_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                };
                lines.push(line);
            }
        }

        if let Some(mut so) = sales_order {
            so.lines = lines.clone();
            Ok(Some((so, lines)))
        } else {
            Ok(None)
        }
    }

    async fn update(&self, sales_order: &SalesOrder) -> Result<(), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Update sales order
        sqlx::query(
            r#"
            UPDATE sales_orders
            SET so_number = $2, customer_id = $3, status = $4, total_amount = $5,
                fulfillment_location_id = $6, updated_at = $7
            WHERE id = $1
            "#,
        )
        .bind(sales_order.id)
        .bind(&sales_order.so_number)
        .bind(sales_order.customer_id)
        .bind(sales_order.status.as_str())
        .bind(sales_order.total_amount)
        .bind(sales_order.fulfillment_location_id)
        .bind(sales_order.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Delete existing lines and re-insert (simplified approach)
        sqlx::query("DELETE FROM sales_order_lines WHERE so_id = $1")
            .bind(sales_order.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert updated lines
        for line in &sales_order.lines {
            sqlx::query(
                r#"
                INSERT INTO sales_order_lines (id, so_id, item_id, qty, unit_price, tax, reserved, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(line.id)
            .bind(line.so_id)
            .bind(line.item_id)
            .bind(line.qty)
            .bind(line.unit_price)
            .bind(line.tax)
            .bind(line.reserved)
            .bind(line.created_at)
            .bind(line.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM sales_orders WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(SalesOrder, Vec<SalesOrderLine>)>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT
                so.id, so.so_number, so.customer_id, so.status, so.total_amount, so.fulfillment_location_id,
                so.created_by, so.created_at, so.updated_at,
                sol.id as line_id, sol.item_id, sol.qty, sol.unit_price, sol.tax, sol.reserved,
                sol.created_at as line_created_at, sol.updated_at as line_updated_at
            FROM sales_orders so
            LEFT JOIN sales_order_lines sol ON so.id = sol.so_id
            ORDER BY so.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let mut result = Vec::new();
        let mut current_so: Option<(SalesOrder, Vec<SalesOrderLine>)> = None;

        for r in rows {
            let so_id: Uuid = r
                .try_get("id")
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

            if current_so.as_ref().map_or(true, |(so, _)| so.id != so_id) {
                // Save previous sales order if exists
                if let Some(so_data) = current_so.take() {
                    result.push(so_data);
                }

                // Start new sales order
                let status_str: String = r
                    .try_get("status")
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                let status = SalesOrderStatus::from_str(&status_str)?;

                current_so = Some((
                    SalesOrder {
                        id: so_id,
                        so_number: r
                            .try_get("so_number")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        customer_id: r
                            .try_get("customer_id")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        status,
                        total_amount: r
                            .try_get("total_amount")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        fulfillment_location_id: r
                            .try_get("fulfillment_location_id")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        lines: Vec::new(),
                        created_by: r
                            .try_get("created_by")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        created_at: r
                            .try_get("created_at")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        updated_at: r
                            .try_get("updated_at")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    },
                    Vec::new(),
                ));
            }

            // Add line if it exists
            if let Ok(line_id) = r.try_get::<Uuid, _>("line_id") {
                if let Some((_, lines)) = current_so.as_mut() {
                    let line = SalesOrderLine {
                        id: line_id,
                        so_id,
                        item_id: r
                            .try_get("item_id")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        qty: r
                            .try_get("qty")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        unit_price: r
                            .try_get("unit_price")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        tax: r
                            .try_get("tax")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        reserved: r
                            .try_get("reserved")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        created_at: r
                            .try_get("line_created_at")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                        updated_at: r
                            .try_get("line_updated_at")
                            .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    };
                    lines.push(line);
                }
            }
        }

        // Add the last sales order
        if let Some(so_data) = current_so.take() {
            result.push(so_data);
        }

        Ok(result)
    }

    async fn ship_sales_order(
        &self,
        id: Uuid,
        shipped_lines: Vec<ShipLineRequest>,
        created_by: Uuid,
    ) -> Result<(SalesOrder, Vec<SalesOrderLine>, Vec<StockMovement>), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Get current sales order
        let (mut sales_order, lines) = self
            .find_by_id_with_tx(&mut tx, id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Sales order {} not found", id)))?;

        // Ship the order (this validates and creates stock movements)
        let stock_movements = sales_order.ship(shipped_lines)?;

        // Update sales order status
        sqlx::query(
            r#"
            UPDATE sales_orders
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(sales_order.id)
        .bind(sales_order.status.as_str())
        .bind(sales_order.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert stock movements
        for movement in &stock_movements {
            sqlx::query(
                r#"
                INSERT INTO stock_movements (id, item_id, location_id, movement_type, quantity, reference_type, reference_id, reason, created_at, created_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(movement.id)
            .bind(movement.item_id)
            .bind(movement.location_id)
            .bind(movement.movement_type.as_str())
            .bind(movement.quantity)
            .bind(movement.reference_type.as_str())
            .bind(movement.reference_id)
            .bind(&movement.reason)
            .bind(movement.created_at)
            .bind(movement.created_by)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((sales_order, lines, stock_movements))
    }

    async fn reserve_inventory(
        &self,
        id: Uuid,
        created_by: Uuid,
    ) -> Result<Vec<StockMovement>, DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Get current sales order
        let (mut sales_order, _) = self
            .find_by_id_with_tx(&mut tx, id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Sales order {} not found", id)))?;

        // Reserve inventory
        let stock_movements = sales_order.reserve_inventory()?;

        // Update line reservations
        for line in &sales_order.lines {
            sqlx::query(
                r#"
                UPDATE sales_order_lines
                SET reserved = $2, updated_at = $3
                WHERE id = $1
                "#,
            )
            .bind(line.id)
            .bind(line.reserved)
            .bind(line.updated_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        // Insert stock movements (reservations)
        for movement in &stock_movements {
            sqlx::query(
                r#"
                INSERT INTO stock_movements (id, item_id, location_id, movement_type, quantity, reference_type, reference_id, reason, created_at, created_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(movement.id)
            .bind(movement.item_id)
            .bind(movement.location_id)
            .bind(movement.movement_type.as_str())
            .bind(movement.quantity)
            .bind(movement.reference_type.as_str())
            .bind(movement.reference_id)
            .bind(&movement.reason)
            .bind(movement.created_at)
            .bind(movement.created_by)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(stock_movements)
    }
}

impl PostgresSalesOrderRepository {
    async fn find_by_id_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
    ) -> Result<Option<(SalesOrder, Vec<SalesOrderLine>)>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT
                so.id, so.so_number, so.customer_id, so.status, so.total_amount, so.fulfillment_location_id,
                so.created_by, so.created_at, so.updated_at,
                sol.id as line_id, sol.item_id, sol.qty, sol.unit_price, sol.tax, sol.reserved,
                sol.created_at as line_created_at, sol.updated_at as line_updated_at
            FROM sales_orders so
            LEFT JOIN sales_order_lines sol ON so.id = sol.so_id
            WHERE so.id = $1
            ORDER BY sol.created_at
            "#,
        )
        .bind(id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if row.is_empty() {
            return Ok(None);
        }

        let mut sales_order: Option<SalesOrder> = None;
        let mut lines = Vec::new();

        for r in row {
            if sales_order.is_none() {
                let status_str: String = r
                    .try_get("status")
                    .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
                let status = SalesOrderStatus::from_str(&status_str)?;

                sales_order = Some(SalesOrder {
                    id: r
                        .try_get("id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    so_number: r
                        .try_get("so_number")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    customer_id: r
                        .try_get("customer_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    status,
                    total_amount: r
                        .try_get("total_amount")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    fulfillment_location_id: r
                        .try_get("fulfillment_location_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    lines: Vec::new(), // Will be set later
                    created_by: r
                        .try_get("created_by")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    updated_at: r
                        .try_get("updated_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                });
            }

            // Add line if it exists
            if let Ok(line_id) = r.try_get::<Uuid, _>("line_id") {
                let line = SalesOrderLine {
                    id: line_id,
                    so_id: r
                        .try_get("id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    item_id: r
                        .try_get("item_id")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    qty: r
                        .try_get("qty")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    unit_price: r
                        .try_get("unit_price")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    tax: r
                        .try_get("tax")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    reserved: r
                        .try_get("reserved")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    created_at: r
                        .try_get("line_created_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                    updated_at: r
                        .try_get("line_updated_at")
                        .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
                };
                lines.push(line);
            }
        }

        if let Some(mut so) = sales_order {
            so.lines = lines.clone();
            Ok(Some((so, lines)))
        } else {
            Ok(None)
        }
    }
}
