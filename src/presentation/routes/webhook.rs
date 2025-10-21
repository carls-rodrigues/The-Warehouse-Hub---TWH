use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::presentation::handlers::webhook::{
    delete_webhook, get_user_webhooks, register_webhook, update_webhook,
};
use crate::AppState;

/// Create webhook-related routes
pub fn create_webhook_routes() -> Router<AppState> {
    Router::new()
        .route("/webhooks", post(register_webhook))
        .route("/webhooks", get(get_user_webhooks))
        .route("/webhooks/{webhook_id}", put(update_webhook))
        .route("/webhooks/{webhook_id}", delete(delete_webhook))
        .layer(CorsLayer::permissive())
}
