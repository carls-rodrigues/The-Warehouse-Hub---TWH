use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::presentation::handlers::stock::{
    adjust_stock, get_item_stock_levels, get_stock_level, get_stock_movements,
};
use crate::AppState;

/// Create stock-related routes
pub fn create_stock_routes() -> Router<AppState> {
    Router::new()
        .route("/stock/{item_id}/{location_id}", get(get_stock_level))
        .route("/stock/items/{item_id}", get(get_item_stock_levels))
        .route("/stock/movements", get(get_stock_movements))
        .route("/stock/adjust", post(adjust_stock))
        .route("/adjustments", post(adjust_stock))
        .layer(CorsLayer::permissive())
}
