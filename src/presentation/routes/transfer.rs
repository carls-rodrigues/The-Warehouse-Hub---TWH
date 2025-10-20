use crate::presentation::handlers::transfer::{
    create_transfer, get_transfer, receive_transfer, ship_transfer,
};
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::AppState;

pub fn transfer_routes() -> Router<AppState> {
    Router::new()
        .route("/transfers", post(create_transfer))
        .route("/transfers/{transferId}", get(get_transfer))
        .route("/transfers/{transferId}/ship", post(ship_transfer))
        .route("/transfers/{transferId}/receive", post(receive_transfer))
        .layer(CorsLayer::permissive())
}
