use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::presentation::handlers::sales_order::{
    create_sales_order, get_sales_order, ship_sales_order,
};
use crate::AppState;

pub fn sales_order_routes() -> Router<AppState> {
    Router::new()
        .route("/sales_orders", post(create_sales_order))
        .route("/sales_orders/{soId}", get(get_sales_order))
        .route("/sales_orders/{soId}/ship", post(ship_sales_order))
        .layer(CorsLayer::permissive())
}
