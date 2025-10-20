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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::search::{
        SearchIndex, SearchIndexRequest, SearchResult, SearchResultItem,
    };
    use async_trait::async_trait;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock SearchRepository for testing
    struct MockSearchRepository {
        search_results: Vec<SearchResultItem>,
        should_fail: bool,
    }

    impl MockSearchRepository {
        fn new() -> Self {
            Self {
                search_results: Vec::new(),
                should_fail: false,
            }
        }

        fn with_results(mut self, results: Vec<SearchResultItem>) -> Self {
            self.search_results = results;
            self
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    #[async_trait]
    impl SearchRepository for MockSearchRepository {
        async fn index_document(&self, _request: SearchIndexRequest) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Indexing failed".to_string()));
            }
            Ok(())
        }

        async fn remove_document(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
        ) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Remove failed".to_string()));
            }
            Ok(())
        }

        async fn search(&self, query: SearchQuery) -> Result<SearchResult, DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Search failed".to_string()));
            }

            // Filter results based on entity types if specified
            let filtered_results = if let Some(ref entity_types) = query.entity_types {
                self.search_results
                    .iter()
                    .filter(|item| entity_types.contains(&item.entity_type))
                    .cloned()
                    .collect()
            } else {
                self.search_results.clone()
            };

            Ok(SearchResult {
                results: filtered_results,
                total: self.search_results.len(),
                query: query.query,
            })
        }

        async fn get_document(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
        ) -> Result<Option<SearchIndex>, DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError(
                    "Get document failed".to_string(),
                ));
            }
            // Mock implementation - return None for simplicity
            Ok(None)
        }

        async fn rebuild_index(&self) -> Result<i64, DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Rebuild failed".to_string()));
            }
            Ok(100) // Mock rebuild count
        }

        async fn cleanup_orphaned_documents(&self) -> Result<i64, DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Cleanup failed".to_string()));
            }
            Ok(5) // Mock cleanup count
        }
    }

    fn create_mock_search_result(
        entity_type: &str,
        entity_id: &str,
        rank: f32,
    ) -> SearchResultItem {
        SearchResultItem {
            entity_type: entity_type.to_string(),
            entity_id: Uuid::new_v4(),
            metadata: Some(serde_json::json!({ "name": entity_id })),
            rank,
        }
    }

    #[tokio::test]
    async fn test_search_all_success() {
        let mock_results = vec![
            create_mock_search_result("item", "widget", 0.9),
            create_mock_search_result("location", "warehouse", 0.8),
            create_mock_search_result("stock_level", "stock-1", 0.7),
        ];

        let mock_repo = Arc::new(MockSearchRepository::new().with_results(mock_results));
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let query = SearchQuery {
            query: "test".to_string(),
            entity_types: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = use_case.search(query).await.unwrap();

        assert_eq!(result.results.len(), 3);
        assert_eq!(result.total, 3);
        assert_eq!(result.query, "test");
    }

    #[tokio::test]
    async fn test_search_all_with_entity_filter() {
        let mock_results = vec![
            create_mock_search_result("item", "widget", 0.9),
            create_mock_search_result("location", "warehouse", 0.8),
            create_mock_search_result("stock_level", "stock-1", 0.7),
        ];

        let mock_repo = Arc::new(MockSearchRepository::new().with_results(mock_results));
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let query = SearchQuery {
            query: "test".to_string(),
            entity_types: Some(vec!["item".to_string()]),
            limit: Some(10),
            offset: Some(0),
        };

        let result = use_case.search(query).await.unwrap();

        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].entity_type, "item");
        assert_eq!(result.total, 3); // Total should still be 3 (all results)
    }

    #[tokio::test]
    async fn test_search_items_success() {
        let mock_results = vec![
            create_mock_search_result("item", "widget", 0.9),
            create_mock_search_result("location", "warehouse", 0.8),
        ];

        let mock_repo = Arc::new(MockSearchRepository::new().with_results(mock_results));
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let query = SearchQuery {
            query: "widget".to_string(),
            entity_types: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = use_case.search_items(query).await.unwrap();

        assert_eq!(result.results.len(), 2); // Mock doesn't filter by entity type
        assert_eq!(result.query, "widget");
    }

    #[tokio::test]
    async fn test_search_locations_success() {
        let mock_results = vec![
            create_mock_search_result("location", "warehouse", 0.8),
            create_mock_search_result("item", "widget", 0.9),
        ];

        let mock_repo = Arc::new(MockSearchRepository::new().with_results(mock_results));
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let query = SearchQuery {
            query: "warehouse".to_string(),
            entity_types: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = use_case.search_locations(query).await.unwrap();

        assert_eq!(result.results.len(), 2); // Mock doesn't filter by entity type
        assert_eq!(result.query, "warehouse");
    }

    #[tokio::test]
    async fn test_search_stock_levels_success() {
        let mock_results = vec![create_mock_search_result("stock_level", "stock-1", 0.7)];

        let mock_repo = Arc::new(MockSearchRepository::new().with_results(mock_results));
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let query = SearchQuery {
            query: "stock".to_string(),
            entity_types: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = use_case.search_stock_levels(query).await.unwrap();

        assert_eq!(result.results.len(), 1);
        assert_eq!(result.query, "stock");
    }

    #[tokio::test]
    async fn test_get_search_suggestions_success() {
        let mock_repo = Arc::new(MockSearchRepository::new());
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let suggestions = use_case
            .get_search_suggestions("test".to_string(), 10)
            .await
            .unwrap();

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "suggestion1");
        assert_eq!(suggestions[1], "suggestion2");
    }

    #[tokio::test]
    async fn test_rebuild_indexes_success() {
        let mock_repo = Arc::new(MockSearchRepository::new());
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let result = use_case.rebuild_indexes().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_failure() {
        let mock_repo = Arc::new(MockSearchRepository::new().with_failure());
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let query = SearchQuery {
            query: "test".to_string(),
            entity_types: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = use_case.search(query).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert_eq!(msg, "Search failed"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_rebuild_indexes_failure() {
        let mock_repo = Arc::new(MockSearchRepository::new().with_failure());
        let use_case = SearchUseCaseImpl::new(mock_repo);

        let result = use_case.rebuild_indexes().await;

        assert!(result.is_err());
    }
}
