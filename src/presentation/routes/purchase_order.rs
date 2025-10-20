use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::presentation::handlers::purchase_order::{
    create_purchase_order, get_purchase_order, receive_purchase_order,
};
use crate::AppState;

/// Create purchase order-related routes
pub fn create_purchase_order_routes() -> Router<AppState> {
    Router::new()
        .route("/purchase_orders", post(create_purchase_order))
        .route("/purchase_orders/{poId}", get(get_purchase_order))
        .route(
            "/purchase_orders/{poId}/receive",
            post(receive_purchase_order),
        )
        .layer(CorsLayer::permissive())
}
