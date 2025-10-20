use crate::domain::entities::transfer::{
    CreateTransferRequest, MovementType, ReceiveTransferRequest, ReferenceType, StockMovement,
    Transfer, TransferLine, TransferStatus,
};
use crate::domain::services::transfer_repository::TransferRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub struct PostgresTransferRepository {
    pool: std::sync::Arc<PgPool>,
}

impl PostgresTransferRepository {
    pub fn new(pool: std::sync::Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn find_by_id_with_tx<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        id: Uuid,
    ) -> Result<Option<(Transfer, Vec<TransferLine>)>, DomainError> {
        // Get transfer
        let transfer_row = sqlx::query!(
            r#"
            SELECT id, transfer_number, from_location_id, to_location_id, status, total_quantity, notes, created_by, created_at, updated_at
            FROM transfers
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let transfer_row = match transfer_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Get transfer lines
        let line_rows = sqlx::query!(
            r#"
            SELECT id, transfer_id, item_id, quantity, quantity_received, created_at, updated_at
            FROM transfer_lines
            WHERE transfer_id = $1
            ORDER BY created_at
            "#,
            id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let lines: Vec<TransferLine> = line_rows
            .into_iter()
            .map(|row| TransferLine {
                id: row.id,
                transfer_id: row.transfer_id,
                item_id: row.item_id,
                quantity: row.quantity,
                quantity_received: row.quantity_received,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        let transfer = Transfer {
            id: transfer_row.id,
            transfer_number: transfer_row.transfer_number,
            from_location_id: transfer_row.from_location_id,
            to_location_id: transfer_row.to_location_id,
            status: TransferStatus::from_str(&transfer_row.status)?,
            total_quantity: transfer_row.total_quantity,
            notes: transfer_row.notes,
            created_by: transfer_row.created_by,
            created_at: transfer_row.created_at,
            updated_at: transfer_row.updated_at,
            lines: lines.clone(),
        };

        Ok(Some((transfer, lines)))
    }
}

#[async_trait]
impl TransferRepository for PostgresTransferRepository {
    async fn create(&self, transfer: &Transfer) -> Result<(), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert transfer
        sqlx::query!(
            r#"
            INSERT INTO transfers (id, transfer_number, from_location_id, to_location_id, status, total_quantity, notes, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            transfer.id,
            transfer.transfer_number,
            transfer.from_location_id,
            transfer.to_location_id,
            transfer.status.as_str(),
            transfer.total_quantity,
            transfer.notes,
            transfer.created_by,
            transfer.created_at,
            transfer.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert transfer lines
        for line in &transfer.lines {
            sqlx::query!(
                r#"
                INSERT INTO transfer_lines (id, transfer_id, item_id, quantity, quantity_received, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                line.id,
                line.transfer_id,
                line.item_id,
                line.quantity,
                line.quantity_received,
                line.created_at,
                line.updated_at
            )
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
    ) -> Result<Option<(Transfer, Vec<TransferLine>)>, DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        self.find_by_id_with_tx(&mut tx, id).await
    }

    async fn find_by_transfer_number(
        &self,
        transfer_number: &str,
    ) -> Result<Option<(Transfer, Vec<TransferLine>)>, DomainError> {
        let transfer_row = sqlx::query!(
            r#"
            SELECT id FROM transfers WHERE transfer_number = $1
            "#,
            transfer_number
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match transfer_row {
            Some(row) => self.find_by_id(row.id).await,
            None => Ok(None),
        }
    }

    async fn update(&self, transfer: &Transfer) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            UPDATE transfers
            SET status = $2, total_quantity = $3, notes = $4, updated_at = $5
            WHERE id = $1
            "#,
            transfer.id,
            transfer.status.as_str(),
            transfer.total_quantity,
            transfer.notes,
            transfer.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM transfers WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Transfer, Vec<TransferLine>)>, DomainError> {
        let transfer_rows = sqlx::query!(
            r#"
            SELECT id FROM transfers
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in transfer_rows {
            if let Some(result) = self.find_by_id(row.id).await? {
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn ship_transfer(
        &self,
        id: Uuid,
        created_by: Uuid,
    ) -> Result<(Transfer, Vec<TransferLine>, Vec<StockMovement>), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Get current transfer
        let (mut transfer, lines) = self
            .find_by_id_with_tx(&mut tx, id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Transfer {} not found", id)))?;

        // Ship the transfer (this validates and creates stock movements)
        let stock_movements = transfer.ship()?;

        // Update transfer status
        sqlx::query!(
            r#"
            UPDATE transfers
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
            transfer.id,
            transfer.status.as_str(),
            transfer.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert stock movements
        for movement in &stock_movements {
            sqlx::query!(
                r#"
                INSERT INTO stock_movements (id, item_id, location_id, movement_type, quantity, reference_type, reference_id, reason, created_at, created_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                movement.id,
                movement.item_id,
                movement.location_id,
                movement.movement_type.as_str(),
                movement.quantity,
                movement.reference_type.as_str(),
                movement.reference_id,
                movement.reason,
                movement.created_at,
                movement.created_by
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((transfer, lines, stock_movements))
    }

    async fn receive_transfer(
        &self,
        id: Uuid,
        received_lines: Vec<crate::domain::entities::transfer::ReceiveTransferLineRequest>,
        created_by: Uuid,
    ) -> Result<(Transfer, Vec<TransferLine>, Vec<StockMovement>), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Get current transfer
        let (mut transfer, mut lines) = self
            .find_by_id_with_tx(&mut tx, id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Transfer {} not found", id)))?;

        // Receive the transfer (this validates and creates stock movements)
        let stock_movements = transfer.receive(received_lines)?;

        // Update transfer status
        sqlx::query!(
            r#"
            UPDATE transfers
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
            transfer.id,
            transfer.status.as_str(),
            transfer.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Update transfer lines
        for line in &transfer.lines {
            sqlx::query!(
                r#"
                UPDATE transfer_lines
                SET quantity_received = $2, updated_at = $3
                WHERE id = $1
                "#,
                line.id,
                line.quantity_received,
                line.updated_at
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        // Insert stock movements
        for movement in &stock_movements {
            sqlx::query!(
                r#"
                INSERT INTO stock_movements (id, item_id, location_id, movement_type, quantity, reference_type, reference_id, reason, created_at, created_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
                movement.id,
                movement.item_id,
                movement.location_id,
                movement.movement_type.as_str(),
                movement.quantity,
                movement.reference_type.as_str(),
                movement.reference_id,
                movement.reason,
                movement.created_at,
                movement.created_by
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        }

        // Get updated lines
        let updated_lines = transfer.lines.clone();

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((transfer, updated_lines, stock_movements))
    }
}
