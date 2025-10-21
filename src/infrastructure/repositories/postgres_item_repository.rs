use crate::domain::entities::item::Item;
use crate::domain::services::item_repository::ItemRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresItemRepository {
    pool: Arc<PgPool>,
}

impl PostgresItemRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ItemRepository for PostgresItemRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Item>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, sku, name, description, category, unit, barcode, cost_price, sale_price,
                   reorder_point, reorder_qty, weight, dimensions, metadata, tenant_id, active, created_at, updated_at
            FROM items
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let dimensions = row
                    .dimensions
                    .map(|d| serde_json::from_value(d).unwrap_or_default());

                Ok(Some(Item {
                    id: row.id,
                    sku: row.sku,
                    name: row.name,
                    description: row.description,
                    category: row.category,
                    unit: row.unit,
                    barcode: row.barcode,
                    cost_price: row.cost_price,
                    sale_price: row.sale_price,
                    reorder_point: row.reorder_point,
                    reorder_qty: row.reorder_qty,
                    weight: row.weight,
                    dimensions,
                    metadata: row.metadata,
                    active: row.active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_sku(&self, sku: &str) -> Result<Option<Item>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, sku, name, description, category, unit, barcode, cost_price, sale_price,
                   reorder_point, reorder_qty, weight, dimensions, metadata, active, created_at, updated_at
            FROM items
            WHERE sku = $1
            "#,
            sku
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let dimensions = row
                    .dimensions
                    .map(|d| serde_json::from_value(d).unwrap_or_default());

                Ok(Some(Item {
                    id: row.id,
                    sku: row.sku,
                    name: row.name,
                    description: row.description,
                    category: row.category,
                    unit: row.unit,
                    barcode: row.barcode,
                    cost_price: row.cost_price,
                    sale_price: row.sale_price,
                    reorder_point: row.reorder_point,
                    reorder_qty: row.reorder_qty,
                    weight: row.weight,
                    dimensions,
                    metadata: row.metadata,
                    active: row.active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, item: &Item) -> Result<(), DomainError> {
        let dimensions_json = item
            .dimensions
            .as_ref()
            .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null));

        sqlx::query!(
            r#"
            INSERT INTO items (id, sku, name, description, category, unit, barcode, cost_price, sale_price,
                              reorder_point, reorder_qty, weight, dimensions, metadata, tenant_id, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
            item.id,
            item.sku,
            item.name,
            item.description,
            item.category,
            item.unit,
            item.barcode,
            item.cost_price,
            item.sale_price,
            item.reorder_point,
            item.reorder_qty,
            item.weight,
            dimensions_json,
            item.metadata,
            uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(), // Default tenant
            item.active,
            item.created_at,
            item.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update(&self, item: &Item) -> Result<(), DomainError> {
        let dimensions_json = item
            .dimensions
            .as_ref()
            .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null));

        sqlx::query!(
            r#"
            UPDATE items
            SET sku = $2, name = $3, description = $4, category = $5, unit = $6, barcode = $7,
                cost_price = $8, sale_price = $9, reorder_point = $10, reorder_qty = $11,
                weight = $12, dimensions = $13, metadata = $14, active = $15, updated_at = $16
            WHERE id = $1
            "#,
            item.id,
            item.sku,
            item.name,
            item.description,
            item.category,
            item.unit,
            item.barcode,
            item.cost_price,
            item.sale_price,
            item.reorder_point,
            item.reorder_qty,
            item.weight,
            dimensions_json,
            item.metadata,
            item.active,
            item.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM items WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Item>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, sku, name, description, category, unit, barcode, cost_price, sale_price,
                   reorder_point, reorder_qty, weight, dimensions, metadata, active, created_at, updated_at
            FROM items
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut items = Vec::new();
        for row in rows {
            let dimensions = row
                .dimensions
                .map(|d| serde_json::from_value(d).unwrap_or_default());

            items.push(Item {
                id: row.id,
                sku: row.sku,
                name: row.name,
                description: row.description,
                category: row.category,
                unit: row.unit,
                barcode: row.barcode,
                cost_price: row.cost_price,
                sale_price: row.sale_price,
                reorder_point: row.reorder_point,
                reorder_qty: row.reorder_qty,
                weight: row.weight,
                dimensions,
                metadata: row.metadata,
                active: row.active,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(items)
    }

    async fn count(&self) -> Result<i64, DomainError> {
        let count: Option<i64> = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM items
            "#
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(count.unwrap_or(0))
    }

    async fn sku_exists(
        &self,
        sku: &str,
        exclude_item_id: Option<Uuid>,
    ) -> Result<bool, DomainError> {
        let count: Option<i64> = if let Some(exclude_id) = exclude_item_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM items WHERE sku = $1 AND id != $2",
                sku,
                exclude_id
            )
            .fetch_one(&*self.pool)
            .await
        } else {
            sqlx::query_scalar!("SELECT COUNT(*) FROM items WHERE sku = $1", sku)
                .fetch_one(&*self.pool)
                .await
        }
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(count.unwrap_or(0) > 0)
    }
}
