use crate::domain::entities::search::{
    SearchIndex, SearchIndexRequest, SearchQuery, SearchResult, SearchResultItem,
};
use crate::shared::error::DomainError;
use async_trait::async_trait;

#[async_trait]
pub trait SearchRepository: Send + Sync {
    /// Index or update a search document
    async fn index_document(&self, request: SearchIndexRequest) -> Result<(), DomainError>;

    /// Remove a document from the search index
    async fn remove_document(
        &self,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<(), DomainError>;

    /// Search documents using full-text search
    async fn search(&self, query: SearchQuery) -> Result<SearchResult, DomainError>;

    /// Get a specific search document
    async fn get_document(
        &self,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<Option<SearchIndex>, DomainError>;

    /// Rebuild search index for all entities (expensive operation)
    async fn rebuild_index(&self) -> Result<i64, DomainError>;

    /// Clean up orphaned search documents
    async fn cleanup_orphaned_documents(&self) -> Result<i64, DomainError>;
}
