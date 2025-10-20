use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::entities::search::{
    SearchIndex, SearchIndexRequest, SearchQuery, SearchResult, SearchResultItem,
};
use crate::domain::services::search_repository::SearchRepository;
use crate::shared::error::DomainError;

#[derive(Clone)]
pub struct PostgresSearchRepository {
    pool: Arc<PgPool>,
}

impl PostgresSearchRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SearchRepository for PostgresSearchRepository {
    async fn index_document(&self, request: SearchIndexRequest) -> Result<(), DomainError> {
        // Use PostgreSQL's to_tsvector function to create the search vector
        let result = sqlx::query(
            r#"
            INSERT INTO search_indexes (entity_type, entity_id, search_vector, metadata, updated_at)
            VALUES ($1, $2, to_tsvector('english', $3), $4, NOW())
            ON CONFLICT (entity_type, entity_id)
            DO UPDATE SET
                search_vector = to_tsvector('english', EXCLUDED.search_vector),
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
        )
        .bind(&request.entity_type)
        .bind(request.entity_id)
        .bind(&request.searchable_content)
        .bind(&request.metadata)
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Failed to index document: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::ValidationError(
                "Failed to index document".to_string(),
            ));
        }

        Ok(())
    }

    async fn remove_document(
        &self,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM search_indexes WHERE entity_type = $1 AND entity_id = $2")
            .bind(entity_type)
            .bind(entity_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                DomainError::ValidationError(format!("Failed to remove document: {}", e))
            })?;

        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<SearchResult, DomainError> {
        let limit = query.limit.unwrap_or(50).min(1000); // Max 1000 results
        let offset = query.offset.unwrap_or(0).max(0);

        let query_builder = if let Some(entity_types) = &query.entity_types {
            if !entity_types.is_empty() {
                let sql = r#"
                    SELECT entity_type, entity_id, ts_rank(search_vector, query) as rank, metadata
                    FROM search_indexes, to_tsquery('english', $1) as query
                    WHERE search_vector @@ query AND entity_type = ANY($2)
                    ORDER BY rank DESC LIMIT $3 OFFSET $4
                "#;
                sqlx::query(sql)
                    .bind(&query.query)
                    .bind(entity_types.as_slice())
                    .bind(limit as i64)
                    .bind(offset as i64)
            } else {
                let sql = r#"
                    SELECT entity_type, entity_id, ts_rank(search_vector, query) as rank, metadata
                    FROM search_indexes, to_tsquery('english', $1) as query
                    WHERE search_vector @@ query
                    ORDER BY rank DESC LIMIT $2 OFFSET $3
                "#;
                sqlx::query(sql)
                    .bind(&query.query)
                    .bind(limit as i64)
                    .bind(offset as i64)
            }
        } else {
            let sql = r#"
                SELECT entity_type, entity_id, ts_rank(search_vector, query) as rank, metadata
                FROM search_indexes, to_tsquery('english', $1) as query
                WHERE search_vector @@ query
                ORDER BY rank DESC LIMIT $2 OFFSET $3
            "#;
            sqlx::query(sql)
                .bind(&query.query)
                .bind(limit as i64)
                .bind(offset as i64)
        };

        let rows = query_builder
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| DomainError::ValidationError(format!("Search failed: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(SearchResultItem {
                entity_type: row
                    .try_get("entity_type")
                    .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                entity_id: row
                    .try_get("entity_id")
                    .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                rank: row
                    .try_get("rank")
                    .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                metadata: row
                    .try_get("metadata")
                    .map_err(|e| DomainError::ValidationError(e.to_string()))?,
            });
        }

        // Get total count for pagination
        let count_sql = r#"
            SELECT COUNT(*) as total
            FROM search_indexes, to_tsquery('english', $1) as query
            WHERE search_vector @@ query
        "#;

        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(&query.query)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| DomainError::ValidationError(format!("Count failed: {}", e)))?;

        Ok(SearchResult {
            query: query.query,
            results,
            total: total as usize,
        })
    }

    async fn get_document(
        &self,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<Option<SearchIndex>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, entity_type, entity_id, search_vector::text, metadata, created_at, updated_at
            FROM search_indexes
            WHERE entity_type = $1 AND entity_id = $2
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Failed to get document: {}", e)))?;

        match row {
            Some(row) => {
                let document = SearchIndex {
                    id: row
                        .try_get("id")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    entity_type: row
                        .try_get("entity_type")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    entity_id: row
                        .try_get("entity_id")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    search_vector: row
                        .try_get("search_vector")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    metadata: row
                        .try_get("metadata")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                    updated_at: row
                        .try_get("updated_at")
                        .map_err(|e| DomainError::ValidationError(e.to_string()))?,
                };
                Ok(Some(document))
            }
            None => Ok(None),
        }
    }

    async fn rebuild_index(&self) -> Result<i64, DomainError> {
        // This is a heavy operation that rebuilds the entire search index
        // In a real implementation, you might want to do this in batches
        let result = sqlx::query("TRUNCATE TABLE search_indexes")
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                DomainError::ValidationError(format!("Failed to truncate search index: {}", e))
            })?;

        // Note: In a real implementation, you would then re-index all entities
        // For now, we just return the number of deleted documents
        Ok(result.rows_affected() as i64)
    }

    async fn cleanup_orphaned_documents(&self) -> Result<i64, DomainError> {
        // Remove search documents for items that no longer exist
        let item_cleanup = sqlx::query(
            r#"
            DELETE FROM search_indexes si
            WHERE si.entity_type = 'item'
            AND NOT EXISTS (SELECT 1 FROM items i WHERE i.id = si.entity_id)
            "#,
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::ValidationError(format!("Failed to cleanup item documents: {}", e))
        })?;

        // Remove search documents for locations that no longer exist
        let location_cleanup = sqlx::query(
            r#"
            DELETE FROM search_indexes si
            WHERE si.entity_type = 'location'
            AND NOT EXISTS (SELECT 1 FROM locations l WHERE l.id = si.entity_id)
            "#,
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| {
            DomainError::ValidationError(format!("Failed to cleanup location documents: {}", e))
        })?;

        Ok(item_cleanup.rows_affected() as i64 + location_cleanup.rows_affected() as i64)
    }
}
