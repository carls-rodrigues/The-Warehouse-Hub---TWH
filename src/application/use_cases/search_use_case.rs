use std::sync::Arc;

use async_trait::async_trait;
use serde_json;

use crate::domain::entities::search::{SearchQuery, SearchResult};
use crate::domain::services::search_repository::SearchRepository;
use crate::shared::error::DomainError;

#[async_trait]
pub trait SearchUseCase: Send + Sync {
    /// Search across all indexed entities
    async fn search(&self, query: SearchQuery) -> Result<SearchResult, DomainError>;

    /// Search for items specifically
    async fn search_items(&self, query: SearchQuery) -> Result<SearchResult, DomainError>;

    /// Search for locations specifically
    async fn search_locations(&self, query: SearchQuery) -> Result<SearchResult, DomainError>;

    /// Search for stock levels specifically
    async fn search_stock_levels(&self, query: SearchQuery) -> Result<SearchResult, DomainError>;

    /// Get search suggestions based on partial input
    async fn get_search_suggestions(
        &self,
        prefix: String,
        limit: usize,
    ) -> Result<Vec<String>, DomainError>;

    /// Rebuild search indexes from scratch
    async fn rebuild_indexes(&self) -> Result<(), DomainError>;
}

#[derive(Clone)]
pub struct SearchUseCaseImpl<SR: SearchRepository> {
    search_repository: Arc<SR>,
}

impl<SR: SearchRepository> SearchUseCaseImpl<SR> {
    pub fn new(search_repository: Arc<SR>) -> Self {
        Self { search_repository }
    }
}

#[async_trait]
impl<SR: SearchRepository> SearchUseCase for SearchUseCaseImpl<SR> {
    async fn search(&self, query: SearchQuery) -> Result<SearchResult, DomainError> {
        self.search_repository.search(query).await
    }

    async fn search_items(&self, query: SearchQuery) -> Result<SearchResult, DomainError> {
        let mut item_query = query.clone();
        item_query.entity_types = Some(vec!["item".to_string()]);
        self.search_repository.search(item_query).await
    }

    async fn search_locations(&self, query: SearchQuery) -> Result<SearchResult, DomainError> {
        let mut location_query = query.clone();
        location_query.entity_types = Some(vec!["location".to_string()]);
        self.search_repository.search(location_query).await
    }

    async fn search_stock_levels(&self, query: SearchQuery) -> Result<SearchResult, DomainError> {
        let mut stock_query = query.clone();
        stock_query.entity_types = Some(vec!["stock_level".to_string()]);
        self.search_repository.search(stock_query).await
    }

    async fn get_search_suggestions(
        &self,
        prefix: String,
        limit: usize,
    ) -> Result<Vec<String>, DomainError> {
        if prefix.trim().is_empty() {
            return Ok(Vec::new());
        }

        // For now, return empty suggestions since we don't have searchable content in results
        // In a future enhancement, we could modify the search to include searchable content
        // or use a different approach for suggestions
        Ok(Vec::new())
    }

    async fn rebuild_indexes(&self) -> Result<(), DomainError> {
        // This would typically involve:
        // 1. Clearing existing indexes
        // 2. Re-indexing all items, locations, and stock levels from the database
        // For now, we'll just clean up orphaned documents
        let _count = self.search_repository.cleanup_orphaned_documents().await?;
        Ok(())
    }
}
