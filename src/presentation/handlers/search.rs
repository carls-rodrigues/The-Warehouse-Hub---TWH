use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

use crate::application::use_cases::search_use_case::SearchUseCase;
use crate::domain::entities::search::SearchQuery;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: String,
    #[serde(rename = "entity_types")]
    entity_types: Option<String>, // comma-separated list
    limit: Option<usize>,
    offset: Option<usize>,
    #[serde(rename = "sort_by")]
    sort_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<ApiSearchResultItem>,
    pub total: usize,
    pub query: String,
    pub took_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiSearchResultItem {
    pub entity_type: String,
    pub entity_id: String,
    pub metadata: Option<serde_json::Value>,
    pub rank: f32,
}

#[derive(Debug, Deserialize)]
pub struct SuggestionsParams {
    prefix: String,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SuggestionsResponse {
    pub suggestions: Vec<String>,
    pub prefix: String,
}

// Convert domain SearchResultItem to API ApiSearchResultItem
impl From<&crate::domain::entities::search::SearchResultItem> for ApiSearchResultItem {
    fn from(item: &crate::domain::entities::search::SearchResultItem) -> Self {
        Self {
            entity_type: item.entity_type.clone(),
            entity_id: item.entity_id.to_string(),
            metadata: item.metadata.clone(),
            rank: item.rank,
        }
    }
}

// Convert API SearchParams to domain SearchQuery
impl From<SearchParams> for SearchQuery {
    fn from(params: SearchParams) -> Self {
        let entity_types = params
            .entity_types
            .map(|types| types.split(',').map(|s| s.trim().to_string()).collect());

        Self {
            query: params.q,
            entity_types,
            limit: params.limit.map(|l| l as i64),
            offset: params.offset.map(|o| o as i64),
        }
    }
}

/// Search across all indexed entities
pub async fn search_all(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = std::time::Instant::now();

    let query: SearchQuery = params.into();
    let result = match state.search_use_case.search(query).await {
        Ok(result) => result,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SearchError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    };

    let took_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchResponse {
        results: result
            .results
            .iter()
            .map(ApiSearchResultItem::from)
            .collect(),
        total: result.total,
        query: result.query,
        took_ms,
    };

    Ok(Json(response))
}

/// Search for items specifically
pub async fn search_items(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = std::time::Instant::now();

    let query: SearchQuery = params.into();
    let result = match state.search_use_case.search_items(query).await {
        Ok(result) => result,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SearchError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    };

    let took_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchResponse {
        results: result
            .results
            .iter()
            .map(ApiSearchResultItem::from)
            .collect(),
        total: result.total,
        query: result.query,
        took_ms,
    };

    Ok(Json(response))
}

/// Search for locations specifically
pub async fn search_locations(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = std::time::Instant::now();

    let query: SearchQuery = params.into();
    let result = match state.search_use_case.search_locations(query).await {
        Ok(result) => result,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SearchError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    };

    let took_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchResponse {
        results: result
            .results
            .iter()
            .map(ApiSearchResultItem::from)
            .collect(),
        total: result.total,
        query: result.query,
        took_ms,
    };

    Ok(Json(response))
}

/// Search for stock levels specifically
pub async fn search_stock_levels(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start_time = std::time::Instant::now();

    let query: SearchQuery = params.into();
    let result = match state.search_use_case.search_stock_levels(query).await {
        Ok(result) => result,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SearchError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    };

    let took_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchResponse {
        results: result
            .results
            .iter()
            .map(ApiSearchResultItem::from)
            .collect(),
        total: result.total,
        query: result.query,
        took_ms,
    };

    Ok(Json(response))
}

/// Get search suggestions
pub async fn get_search_suggestions(
    State(state): State<AppState>,
    Query(params): Query<SuggestionsParams>,
) -> Result<Json<SuggestionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit.unwrap_or(10).min(50); // Cap at 50 suggestions
    let suggestions = match state
        .search_use_case
        .get_search_suggestions(params.prefix.clone(), limit)
        .await
    {
        Ok(suggestions) => suggestions,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SearchError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    };

    let response = SuggestionsResponse {
        suggestions,
        prefix: params.prefix,
    };

    Ok(Json(response))
}

/// Rebuild search indexes (admin endpoint)
pub async fn rebuild_search_indexes(
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.search_use_case.rebuild_indexes().await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "RebuildError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
