use crate::shared::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub search_vector: String, // TSVECTOR as string representation
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndexRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub searchable_content: String, // Raw content to be converted to TSVECTOR
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<SearchResultItem>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub rank: f32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub entity_types: Option<Vec<String>>, // Filter by entity types
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl SearchIndex {
    pub fn new(request: SearchIndexRequest) -> Result<Self, DomainError> {
        let now = Utc::now();

        if request.entity_type.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Entity type cannot be empty".to_string(),
            ));
        }

        if request.searchable_content.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Searchable content cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            entity_type: request.entity_type,
            entity_id: request.entity_id,
            search_vector: request.searchable_content, // Will be processed by PostgreSQL
            metadata: request.metadata,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_content(
        &mut self,
        content: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), DomainError> {
        if content.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Searchable content cannot be empty".to_string(),
            ));
        }

        self.search_vector = content;
        self.metadata = metadata;
        self.updated_at = Utc::now();
        Ok(())
    }
}
