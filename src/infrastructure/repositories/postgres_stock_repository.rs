use crate::domain::entities::inventory::{MovementType, ReferenceType, StockLevel, StockMovement};
use crate::domain::services::stock_repository::StockRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresStockRepository {
    pool: Arc<PgPool>,
}

impl PostgresStockRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Execute stock movement and level update in a single transaction
    async fn execute_movement_transaction(
        &self,
        movement: &StockMovement,
    ) -> Result<(), DomainError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            DomainError::ValidationError(format!("Failed to start transaction: {}", e))
        })?;

        // Insert the stock movement
        sqlx::query!(
            r#"
            INSERT INTO stock_movements (
                id, item_id, location_id, movement_type, quantity,
                reference_type, reference_id, reason, created_at, created_by, tenant_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, get_current_tenant_id())
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
        .map_err(|e| {
            DomainError::ValidationError(format!("Failed to insert stock movement: {}", e))
        })?;

        // Update or insert stock level
        sqlx::query!(
            r#"
            INSERT INTO stock_levels (item_id, location_id, quantity_on_hand, last_movement_id, updated_at, tenant_id)
            VALUES ($1, $2, $3, $4, $5, get_current_tenant_id())
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
        .map_err(|e| DomainError::ValidationError(format!("Failed to update stock level: {}", e)))?;

        // Validate that stock level is not negative (except for adjustments)
        if movement.movement_type != MovementType::Adjustment {
            let stock_level = sqlx::query!(
                r#"SELECT quantity_on_hand FROM stock_levels WHERE item_id = $1 AND location_id = $2"#,
                movement.item_id,
                movement.location_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| DomainError::ValidationError(format!("Failed to check stock level: {}", e)))?;

            if stock_level.quantity_on_hand < 0 {
                tx.rollback().await.map_err(|e| {
                    DomainError::ValidationError(format!("Failed to rollback transaction: {}", e))
                })?;
                return Err(DomainError::BusinessLogicError(
                    "Stock level cannot go negative".to_string(),
                ));
            }
        }

        tx.commit().await.map_err(|e| {
            DomainError::ValidationError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }
}

#[async_trait]
impl StockRepository for PostgresStockRepository {
    async fn record_movement(&self, movement: &StockMovement) -> Result<(), DomainError> {
        self.execute_movement_transaction(movement).await
    }

    async fn get_stock_level(
        &self,
        item_id: Uuid,
        location_id: Uuid,
    ) -> Result<Option<StockLevel>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT item_id, location_id, quantity_on_hand, last_movement_id, updated_at
            FROM stock_levels
            WHERE item_id = $1 AND location_id = $2 AND tenant_id = get_current_tenant_id()
            "#,
            item_id,
            location_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(result.map(|row| StockLevel {
            item_id: row.item_id,
            location_id: row.location_id,
            quantity_on_hand: row.quantity_on_hand,
            last_movement_id: row.last_movement_id,
            updated_at: row.updated_at,
        }))
    }

    async fn get_item_stock_levels(&self, item_id: Uuid) -> Result<Vec<StockLevel>, DomainError> {
        let results = sqlx::query!(
            r#"
            SELECT item_id, location_id, quantity_on_hand, last_movement_id, updated_at
            FROM stock_levels
            WHERE item_id = $1 AND tenant_id = get_current_tenant_id()
            ORDER BY location_id
            "#,
            item_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(results
            .into_iter()
            .map(|row| StockLevel {
                item_id: row.item_id,
                location_id: row.location_id,
                quantity_on_hand: row.quantity_on_hand,
                last_movement_id: row.last_movement_id,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn get_location_stock_levels(
        &self,
        location_id: Uuid,
    ) -> Result<Vec<StockLevel>, DomainError> {
        let results = sqlx::query!(
            r#"
            SELECT item_id, location_id, quantity_on_hand, last_movement_id, updated_at
            FROM stock_levels
            WHERE location_id = $1 AND tenant_id = get_current_tenant_id()
            ORDER BY item_id
            "#,
            location_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(results
            .into_iter()
            .map(|row| StockLevel {
                item_id: row.item_id,
                location_id: row.location_id,
                quantity_on_hand: row.quantity_on_hand,
                last_movement_id: row.last_movement_id,
                updated_at: row.updated_at,
            })
            .collect())
    }

    async fn get_item_movements(
        &self,
        item_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovement>, DomainError> {
        let results = sqlx::query!(
            r#"
            SELECT id, item_id, location_id, movement_type, quantity,
                   reference_type, reference_id, reason, created_at, created_by
            FROM stock_movements
            WHERE item_id = $1 AND tenant_id = get_current_tenant_id()
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            item_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut movements = Vec::new();
        for row in results {
            let movement_type = MovementType::from_str(&row.movement_type)?;
            let reference_type = ReferenceType::from_str(&row.reference_type)?;

            movements.push(StockMovement {
                id: row.id,
                item_id: row.item_id,
                location_id: row.location_id,
                movement_type,
                quantity: row.quantity,
                reference_type,
                reference_id: row.reference_id,
                reason: row.reason,
                created_at: row.created_at,
                created_by: row.created_by,
            });
        }

        Ok(movements)
    }

    async fn get_location_movements(
        &self,
        location_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovement>, DomainError> {
        let results = sqlx::query!(
            r#"
            SELECT id, item_id, location_id, movement_type, quantity,
                   reference_type, reference_id, reason, created_at, created_by
            FROM stock_movements
            WHERE location_id = $1 AND tenant_id = get_current_tenant_id()
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            location_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut movements = Vec::new();
        for row in results {
            let movement_type = MovementType::from_str(&row.movement_type)?;
            let reference_type = ReferenceType::from_str(&row.reference_type)?;

            movements.push(StockMovement {
                id: row.id,
                item_id: row.item_id,
                location_id: row.location_id,
                movement_type,
                quantity: row.quantity,
                reference_type,
                reference_id: row.reference_id,
                reason: row.reason,
                created_at: row.created_at,
                created_by: row.created_by,
            });
        }

        Ok(movements)
    }

    async fn get_stock_movements(
        &self,
        item_id: Uuid,
        location_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StockMovement>, DomainError> {
        let results = sqlx::query!(
            r#"
            SELECT id, item_id, location_id, movement_type, quantity,
                   reference_type, reference_id, reason, created_at, created_by
            FROM stock_movements
            WHERE item_id = $1 AND location_id = $2 AND tenant_id = get_current_tenant_id()
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            item_id,
            location_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut movements = Vec::new();
        for row in results {
            let movement_type = MovementType::from_str(&row.movement_type)?;
            let reference_type = ReferenceType::from_str(&row.reference_type)?;

            movements.push(StockMovement {
                id: row.id,
                item_id: row.item_id,
                location_id: row.location_id,
                movement_type,
                quantity: row.quantity,
                reference_type,
                reference_id: row.reference_id,
                reason: row.reason,
                created_at: row.created_at,
                created_by: row.created_by,
            });
        }

        Ok(movements)
    }

    async fn get_movement_by_id(&self, id: Uuid) -> Result<Option<StockMovement>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, item_id, location_id, movement_type, quantity,
                   reference_type, reference_id, reason, created_at, created_by
            FROM stock_movements
            WHERE id = $1 AND tenant_id = get_current_tenant_id()
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let movement_type = MovementType::from_str(&row.movement_type)?;
                let reference_type = ReferenceType::from_str(&row.reference_type)?;

                Ok(Some(StockMovement {
                    id: row.id,
                    item_id: row.item_id,
                    location_id: row.location_id,
                    movement_type,
                    quantity: row.quantity,
                    reference_type,
                    reference_id: row.reference_id,
                    reason: row.reason,
                    created_at: row.created_at,
                    created_by: row.created_by,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_total_quantity_on_hand(&self, item_id: Uuid) -> Result<i32, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(quantity_on_hand), 0) as total
            FROM stock_levels
            WHERE item_id = $1 AND tenant_id = get_current_tenant_id()
            "#,
            item_id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(result.total.unwrap_or(0) as i32)
    }

    async fn initialize_stock_level(
        &self,
        item_id: Uuid,
        location_id: Uuid,
    ) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            INSERT INTO stock_levels (item_id, location_id, quantity_on_hand, updated_at, tenant_id)
            VALUES ($1, $2, 0, NOW(), get_current_tenant_id())
            ON CONFLICT (item_id, location_id) DO NOTHING
            "#,
            item_id,
            location_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn stock_level_exists(
        &self,
        item_id: Uuid,
        location_id: Uuid,
    ) -> Result<bool, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM stock_levels
                WHERE item_id = $1 AND location_id = $2 AND tenant_id = get_current_tenant_id()
            ) as exists
            "#,
            item_id,
            location_id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(result.exists.unwrap_or(false))
    }

    async fn get_stock_levels_below_threshold(
        &self,
        threshold: i32,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<crate::domain::services::stock_repository::PaginatedStockLevels, DomainError> {
        let offset = cursor
            .as_ref()
            .and_then(|c| c.parse::<i64>().ok())
            .unwrap_or(0);

        let results: Vec<_> = sqlx::query!(
            r#"
            SELECT item_id, location_id, quantity_on_hand, last_movement_id, updated_at
            FROM stock_levels
            WHERE quantity_on_hand <= $1 AND tenant_id = get_current_tenant_id()
            ORDER BY item_id, location_id
            LIMIT $2 OFFSET $3
            "#,
            threshold,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let stock_levels: Vec<StockLevel> = results
            .into_iter()
            .map(|row| StockLevel {
                item_id: row.item_id,
                location_id: row.location_id,
                quantity_on_hand: row.quantity_on_hand,
                last_movement_id: row.last_movement_id,
                updated_at: row.updated_at,
            })
            .collect();

        let next_cursor = if stock_levels.len() == limit as usize {
            Some((offset + limit).to_string())
        } else {
            None
        };

        Ok(
            crate::domain::services::stock_repository::PaginatedStockLevels {
                items: stock_levels,
                next_cursor,
            },
        )
    }

    async fn get_stock_levels_by_location(
        &self,
        location_id: Uuid,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<crate::domain::services::stock_repository::PaginatedStockLevels, DomainError> {
        let offset = cursor
            .as_ref()
            .and_then(|c| c.parse::<i64>().ok())
            .unwrap_or(0);

        let results: Vec<_> = sqlx::query!(
            r#"
            SELECT item_id, location_id, quantity_on_hand, last_movement_id, updated_at
            FROM stock_levels
            WHERE location_id = $1 AND tenant_id = get_current_tenant_id()
            ORDER BY item_id
            LIMIT $2 OFFSET $3
            "#,
            location_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let stock_levels: Vec<StockLevel> = results
            .into_iter()
            .map(|row| StockLevel {
                item_id: row.item_id,
                location_id: row.location_id,
                quantity_on_hand: row.quantity_on_hand,
                last_movement_id: row.last_movement_id,
                updated_at: row.updated_at,
            })
            .collect();

        let next_cursor = if stock_levels.len() == limit as usize {
            Some((offset + limit).to_string())
        } else {
            None
        };

        Ok(
            crate::domain::services::stock_repository::PaginatedStockLevels {
                items: stock_levels,
                next_cursor,
            },
        )
    }

    async fn get_all_stock_levels(
        &self,
        limit: i64,
        cursor: Option<String>,
    ) -> Result<crate::domain::services::stock_repository::PaginatedStockLevels, DomainError> {
        let offset = cursor
            .as_ref()
            .and_then(|c| c.parse::<i64>().ok())
            .unwrap_or(0);

        let results: Vec<_> = sqlx::query!(
            r#"
            SELECT item_id, location_id, quantity_on_hand, last_movement_id, updated_at
            FROM stock_levels
            WHERE tenant_id = get_current_tenant_id()
            ORDER BY item_id, location_id
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let stock_levels: Vec<StockLevel> = results
            .into_iter()
            .map(|row| StockLevel {
                item_id: row.item_id,
                location_id: row.location_id,
                quantity_on_hand: row.quantity_on_hand,
                last_movement_id: row.last_movement_id,
                updated_at: row.updated_at,
            })
            .collect();

        let next_cursor = if stock_levels.len() == limit as usize {
            Some((offset + limit).to_string())
        } else {
            None
        };

        Ok(
            crate::domain::services::stock_repository::PaginatedStockLevels {
                items: stock_levels,
                next_cursor,
            },
        )
    }
}
