use crate::domain::entities::inventory::StockMovement;
use crate::domain::entities::purchase_order::{
    CreatePurchaseOrderRequest, PurchaseOrder, PurchaseOrderLine, PurchaseOrderStatus,
    ReceivePurchaseOrderRequest,
};
use crate::domain::services::purchase_order_repository::PurchaseOrderRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresPurchaseOrderRepository {
    pool: Arc<PgPool>,
}

impl PostgresPurchaseOrderRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseOrderRepository for PostgresPurchaseOrderRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<PurchaseOrder>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT
                po.id, po.po_number, po.supplier_id, po.status, po.expected_date,
                po.total_amount, po.created_by, po.created_at, po.updated_at,
                pol.id as line_id, pol.item_id, pol.qty_ordered, pol.qty_received,
                pol.unit_cost, pol.line_total
            FROM purchase_orders po
            LEFT JOIN purchase_order_lines pol ON po.id = pol.po_id
            WHERE po.id = $1
            ORDER BY pol.created_at
            "#,
            id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        if result.is_empty() {
            return Ok(None);
        }

        // Extract header info from first row
        let po_id = result[0].id;
        let po_number = result[0].po_number.clone();
        let supplier_id = result[0].supplier_id;
        let status_str = result[0].status.as_str();
        let expected_date = result[0].expected_date;
        let total_amount = result[0].total_amount;
        let created_by = result[0].created_by;
        let created_at = result[0].created_at;
        let updated_at = result[0].updated_at;

        let status = match status_str {
            "DRAFT" => PurchaseOrderStatus::Draft,
            "OPEN" => PurchaseOrderStatus::Open,
            "RECEIVING" => PurchaseOrderStatus::Receiving,
            "PARTIAL_RECEIVED" => PurchaseOrderStatus::PartialReceived,
            "RECEIVED" => PurchaseOrderStatus::Received,
            "CANCELLED" => PurchaseOrderStatus::Cancelled,
            _ => {
                return Err(DomainError::InfrastructureError(
                    "Invalid status".to_string(),
                ))
            }
        };

        let mut lines = Vec::new();
        for row in result {
            lines.push(PurchaseOrderLine {
                id: row.line_id,
                po_id: id,
                item_id: row.item_id,
                qty_ordered: row.qty_ordered,
                qty_received: row.qty_received,
                unit_cost: row.unit_cost,
                line_total: row.line_total,
            });
        }

        Ok(Some(PurchaseOrder {
            id: po_id,
            po_number,
            supplier_id,
            status,
            expected_date,
            total_amount,
            lines,
            created_by,
            created_at,
            updated_at,
        }))
    }

    async fn find_by_po_number(
        &self,
        po_number: &str,
    ) -> Result<Option<PurchaseOrder>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id FROM purchase_orders WHERE po_number = $1
            "#,
            po_number
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => self.find_by_id(row.id).await,
            None => Ok(None),
        }
    }

    async fn save(&self, po: &PurchaseOrder) -> Result<(), DomainError> {
        let status_str = match po.status {
            PurchaseOrderStatus::Draft => "DRAFT",
            PurchaseOrderStatus::Open => "OPEN",
            PurchaseOrderStatus::Receiving => "RECEIVING",
            PurchaseOrderStatus::PartialReceived => "PARTIAL_RECEIVED",
            PurchaseOrderStatus::Received => "RECEIVED",
            PurchaseOrderStatus::Cancelled => "CANCELLED",
        };

        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DomainError::InfrastructureError(format!("Transaction error: {}", e))
            })?;

        sqlx::query!(
            r#"
            INSERT INTO purchase_orders (id, po_number, supplier_id, status, expected_date, total_amount, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            po.id,
            po.po_number,
            po.supplier_id,
            status_str,
            po.expected_date,
            po.total_amount,
            po.created_by,
            po.created_at,
            po.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        for line in &po.lines {
            sqlx::query!(
                r#"
                INSERT INTO purchase_order_lines (id, po_id, item_id, qty_ordered, qty_received, unit_cost, line_total, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                line.id,
                line.po_id,
                line.item_id,
                line.qty_ordered,
                line.qty_received,
                line.unit_cost,
                line.line_total,
                po.created_at,
                po.updated_at
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;
        }

        tx.commit().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Transaction commit error: {}", e))
        })?;

        Ok(())
    }

    async fn update(&self, po: &PurchaseOrder) -> Result<(), DomainError> {
        let status_str = match po.status {
            PurchaseOrderStatus::Draft => "DRAFT",
            PurchaseOrderStatus::Open => "OPEN",
            PurchaseOrderStatus::Receiving => "RECEIVING",
            PurchaseOrderStatus::PartialReceived => "PARTIAL_RECEIVED",
            PurchaseOrderStatus::Received => "RECEIVED",
            PurchaseOrderStatus::Cancelled => "CANCELLED",
        };

        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DomainError::InfrastructureError(format!("Transaction error: {}", e))
            })?;

        sqlx::query!(
            r#"
            UPDATE purchase_orders
            SET status = $2, expected_date = $3, total_amount = $4, updated_at = $5
            WHERE id = $1
            "#,
            po.id,
            status_str,
            po.expected_date,
            po.total_amount,
            po.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        // Update lines
        for line in &po.lines {
            sqlx::query!(
                r#"
                UPDATE purchase_order_lines
                SET qty_received = $2, updated_at = $3
                WHERE id = $1
                "#,
                line.id,
                line.qty_received,
                po.updated_at
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;
        }

        tx.commit().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Transaction commit error: {}", e))
        })?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM purchase_orders WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        status_filter: Option<String>,
    ) -> Result<Vec<PurchaseOrder>, DomainError> {
        let rows = if let Some(status) = &status_filter {
            sqlx::query("SELECT id FROM purchase_orders WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(status)
                .bind(limit)
                .bind(offset)
                .fetch_all(&*self.pool)
                .await
        } else {
            sqlx::query("SELECT id FROM purchase_orders ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&*self.pool)
                .await
        }
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        let mut pos = Vec::new();
        for row in rows {
            let id: Uuid = row.get("id");
            if let Some(po) = self.find_by_id(id).await? {
                pos.push(po);
            }
        }

        Ok(pos)
    }

    async fn count(&self, status_filter: Option<String>) -> Result<i64, DomainError> {
        let result = if let Some(status) = &status_filter {
            sqlx::query("SELECT COUNT(*) as count FROM purchase_orders WHERE status = $1")
                .bind(status)
                .fetch_one(&*self.pool)
                .await
        } else {
            sqlx::query("SELECT COUNT(*) as count FROM purchase_orders")
                .fetch_one(&*self.pool)
                .await
        }
        .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

        Ok(result.get::<i64, _>("count"))
    }

    async fn receive_purchase_order(
        &self,
        po_id: Uuid,
        request: &ReceivePurchaseOrderRequest,
        user_id: Uuid,
    ) -> Result<Vec<StockMovement>, DomainError> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DomainError::InfrastructureError(format!("Transaction error: {}", e))
            })?;

        // Get current PO
        let Some(mut po) = self.find_by_id(po_id).await? else {
            return Err(DomainError::ValidationError(
                "Purchase order not found".to_string(),
            ));
        };

        // Receive the lines
        po.receive_lines(request.received_lines.clone())?;

        // Update PO
        self.update(&po).await?;

        // Create stock movements for received items
        let mut movements = Vec::new();
        for receive_req in &request.received_lines {
            if receive_req.qty_received > 0 {
                let line = po
                    .lines
                    .iter()
                    .find(|l| l.id == receive_req.po_line_id)
                    .ok_or_else(|| DomainError::ValidationError("Line not found".to_string()))?;

                let movement = StockMovement::new(
                    line.item_id,
                    request.destination_location_id,
                    crate::domain::entities::inventory::MovementType::Inbound,
                    receive_req.qty_received,
                    crate::domain::entities::inventory::ReferenceType::PurchaseOrder,
                    Some(po_id),
                    Some(format!("PO-{}", po.po_number)),
                    Some(user_id),
                )?;

                // Save movement
                sqlx::query!(
                    r#"
                    INSERT INTO stock_movements (id, item_id, location_id, quantity, movement_type, reference_type, reference_id, reason, created_by, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    "#,
                    movement.id,
                    movement.item_id,
                    movement.location_id,
                    movement.quantity,
                    "inbound", // Map to database enum
                    movement.reference_type.as_str(),
                    movement.reference_id,
                    movement.reason,
                    movement.created_by,
                    movement.created_at
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

                // Update stock levels
                sqlx::query!(
                    r#"
                    INSERT INTO stock_levels (item_id, location_id, quantity_on_hand, last_movement_id, updated_at)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (item_id, location_id)
                    DO UPDATE SET
                        quantity_on_hand = stock_levels.quantity_on_hand + EXCLUDED.quantity_on_hand,
                        last_movement_id = EXCLUDED.last_movement_id,
                        updated_at = EXCLUDED.updated_at
                    "#,
                    movement.item_id,
                    movement.location_id,
                    movement.quantity,
                    movement.id,
                    movement.created_at
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::InfrastructureError(format!("Database error: {}", e)))?;

                movements.push(movement);
            }
        }

        tx.commit().await.map_err(|e| {
            DomainError::InfrastructureError(format!("Transaction commit error: {}", e))
        })?;

        Ok(movements)
    }
}
