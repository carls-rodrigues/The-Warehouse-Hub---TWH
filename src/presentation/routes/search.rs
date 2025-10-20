use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::application::use_cases::search_use_case::{SearchUseCase, SearchUseCaseImpl};
use crate::infrastructure::repositories::postgres_search_repository::PostgresSearchRepository;
use crate::presentation::handlers::search::{
    get_search_suggestions, rebuild_search_indexes, search_all, search_items, search_locations,
    search_stock_levels,
};
use crate::AppState;

/// Create search routes
pub fn create_search_routes() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_all))
        .route("/search/items", get(search_items))
        .route("/search/locations", get(search_locations))
        .route("/search/stock-levels", get(search_stock_levels))
        .route("/search/suggestions", get(get_search_suggestions))
        .route("/admin/search/rebuild", post(rebuild_search_indexes))
}
