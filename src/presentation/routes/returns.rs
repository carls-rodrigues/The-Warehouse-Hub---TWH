use crate::presentation::handlers::returns::{
    create_return, get_return, open_return, process_return,
};
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::AppState;

pub fn return_routes() -> Router<AppState> {
    Router::new()
        .route("/returns", post(create_return))
        .route("/returns/{returnId}", get(get_return))
        .route("/returns/{returnId}/open", post(open_return))
        .route("/returns/{returnId}/process", post(process_return))
        .layer(CorsLayer::permissive())
}
