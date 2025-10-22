use axum::{
    routing::{get, post, put},
    Router,
};

use crate::presentation::handlers::admin::{
    admin_dashboard_handler, cleanup_expired_sandboxes_handler, get_billing_metrics_handler,
    get_tenant_quotas_handler, list_dlq_deliveries_handler, list_sandboxes_handler,
    replay_dlq_delivery_handler, update_tenant_quotas_handler,
};
use crate::AppState;

pub fn create_admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/dashboard", get(admin_dashboard_handler))
        .route("/admin/sandboxes", get(list_sandboxes_handler))
        .route(
            "/admin/sandboxes/cleanup",
            post(cleanup_expired_sandboxes_handler),
        )
        .route("/admin/dlq", get(list_dlq_deliveries_handler))
        .route("/admin/dlq/replay", post(replay_dlq_delivery_handler))
        .route("/admin/billing", get(get_billing_metrics_handler))
        .route(
            "/admin/tenants/{tenant_id}/quotas",
            get(get_tenant_quotas_handler),
        )
        .route(
            "/admin/tenants/{tenant_id}/quotas",
            put(update_tenant_quotas_handler),
        )
}
