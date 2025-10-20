use crate::domain::entities::inventory::StockMovement;
use crate::domain::entities::returns::{ProcessReturnRequest, Return, ReturnLine, ReturnStatus};
use crate::domain::services::return_repository::ReturnRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub struct PostgresReturnRepository {
    pool: std::sync::Arc<PgPool>,
}

impl PostgresReturnRepository {
    pub fn new(pool: std::sync::Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn find_by_id_with_tx<'a>(
        &self,
        tx: &mut Transaction<'a, Postgres>,
        id: Uuid,
    ) -> Result<Option<(Return, Vec<ReturnLine>)>, DomainError> {
        // Get return
        let return_row = sqlx::query!(
            r#"
            SELECT id, return_number, location_id, customer_id, status, total_quantity, notes, created_by, created_at, updated_at
            FROM returns
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let return_row = match return_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Get return lines
        let line_rows = sqlx::query!(
            r#"
            SELECT id, return_id, item_id, quantity, quantity_received, unit_price, reason, created_at, updated_at
            FROM return_lines
            WHERE return_id = $1
            ORDER BY created_at
            "#,
            id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let return_entity = Return {
            id: return_row.id,
            return_number: return_row.return_number,
            location_id: return_row.location_id,
            customer_id: return_row.customer_id,
            status: ReturnStatus::from_str(&return_row.status)
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?,
            total_quantity: return_row.total_quantity,
            notes: return_row.notes,
            lines: vec![], // Will be filled below
            created_by: return_row.created_by,
            created_at: return_row.created_at,
            updated_at: return_row.updated_at,
        };

        let lines: Vec<ReturnLine> = line_rows
            .into_iter()
            .map(|row| ReturnLine {
                id: row.id,
                return_id: row.return_id,
                item_id: row.item_id,
                quantity: row.quantity,
                quantity_received: row.quantity_received,
                unit_price: row.unit_price,
                reason: row.reason,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(Some((return_entity, lines)))
    }
}

#[async_trait]
impl ReturnRepository for PostgresReturnRepository {
    async fn create(&self, return_entity: &Return) -> Result<(), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert return
        sqlx::query!(
            r#"
            INSERT INTO returns (id, return_number, location_id, customer_id, status, total_quantity, notes, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            return_entity.id,
            return_entity.return_number,
            return_entity.location_id,
            return_entity.customer_id,
            return_entity.status.as_str(),
            return_entity.total_quantity,
            return_entity.notes,
            return_entity.created_by,
            return_entity.created_at,
            return_entity.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Insert return lines
        for line in &return_entity.lines {
            sqlx::query!(
                r#"
                INSERT INTO return_lines (id, return_id, item_id, quantity, quantity_received, unit_price, reason, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                line.id,
                line.return_id,
                line.item_id,
                line.quantity,
                line.quantity_received,
                line.unit_price,
                line.reason,
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

    async fn find_by_id(&self, id: Uuid) -> Result<Option<(Return, Vec<ReturnLine>)>, DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        self.find_by_id_with_tx(&mut tx, id).await
    }

    async fn find_by_return_number(
        &self,
        return_number: &str,
    ) -> Result<Option<(Return, Vec<ReturnLine>)>, DomainError> {
        let return_row = sqlx::query!(
            r#"
            SELECT id FROM returns WHERE return_number = $1
            "#,
            return_number
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        match return_row {
            Some(row) => self.find_by_id(row.id).await,
            None => Ok(None),
        }
    }

    async fn update(&self, return_entity: &Return) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            UPDATE returns
            SET return_number = $2, location_id = $3, customer_id = $4, status = $5, total_quantity = $6, notes = $7, updated_at = $8
            WHERE id = $1
            "#,
            return_entity.id,
            return_entity.return_number,
            return_entity.location_id,
            return_entity.customer_id,
            return_entity.status.as_str(),
            return_entity.total_quantity,
            return_entity.notes,
            return_entity.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM returns WHERE id = $1
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
    ) -> Result<Vec<(Return, Vec<ReturnLine>)>, DomainError> {
        let return_rows = sqlx::query!(
            r#"
            SELECT id FROM returns
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
        for row in return_rows {
            if let Some(result) = self.find_by_id(row.id).await? {
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn open_return(&self, id: Uuid) -> Result<(Return, Vec<ReturnLine>), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Get current return
        let (mut return_entity, lines) = self
            .find_by_id_with_tx(&mut tx, id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Return {} not found", id)))?;

        // Open the return (this validates status transition)
        return_entity.open()?;

        // Update return status
        sqlx::query!(
            r#"
            UPDATE returns
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
            return_entity.id,
            return_entity.status.as_str(),
            return_entity.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((return_entity, lines))
    }

    async fn process_return(
        &self,
        id: Uuid,
        process_request: ProcessReturnRequest,
        created_by: Uuid,
    ) -> Result<(Return, Vec<ReturnLine>, Vec<StockMovement>), DomainError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Get current return
        let (mut return_entity, mut lines) = self
            .find_by_id_with_tx(&mut tx, id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Return {} not found", id)))?;

        // Process the return (this validates and creates stock movements)
        let stock_movements = return_entity.process(process_request.lines.clone())?;

        // Update return status and lines
        sqlx::query!(
            r#"
            UPDATE returns
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
            return_entity.id,
            return_entity.status.as_str(),
            return_entity.updated_at
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Update return lines with received quantities
        for line in &return_entity.lines {
            sqlx::query!(
                r#"
                UPDATE return_lines
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

        tx.commit()
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok((return_entity, lines, stock_movements))
    }
}
