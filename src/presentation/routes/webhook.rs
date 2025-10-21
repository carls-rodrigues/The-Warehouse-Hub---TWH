use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::presentation::handlers::webhook::{
    delete_webhook, get_user_webhooks, register_webhook, update_webhook,
};
use crate::presentation::handlers::webhook_deliveries::{
    get_webhook_deliveries, get_webhook_delivery_details, retry_webhook_delivery, test_webhook,
};
use crate::AppState;

/// Create webhook-related routes
pub fn create_webhook_routes() -> Router<AppState> {
    Router::new()
        .route("/webhooks", post(register_webhook))
        .route("/webhooks", get(get_user_webhooks))
        .route("/webhooks/{webhook_id}", put(update_webhook))
        .route("/webhooks/{webhook_id}", delete(delete_webhook))
        .route(
            "/webhooks/{webhook_id}/deliveries",
            get(get_webhook_deliveries),
        )
        .route(
            "/webhooks/deliveries/{delivery_id}",
            get(get_webhook_delivery_details),
        )
        .route("/webhooks/{webhook_id}/test", post(test_webhook))
        .route(
            "/webhooks/deliveries/{delivery_id}/retry",
            post(retry_webhook_delivery),
        )
        .layer(CorsLayer::permissive())
}
