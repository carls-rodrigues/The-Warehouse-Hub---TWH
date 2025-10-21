use crate::presentation::handlers::tenant::{
    cleanup_expired_sandboxes, create_sandbox_tenant, create_tenant, delete_tenant, get_tenant,
    list_tenants,
};
use crate::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn tenant_routes() -> Router<AppState> {
    Router::new()
        .route("/tenants", post(create_tenant))
        .route("/tenants/sandbox", post(create_sandbox_tenant))
        .route("/tenants", get(list_tenants))
        .route("/tenants/cleanup", post(cleanup_expired_sandboxes))
        .route("/tenants/{tenant_id}", get(get_tenant))
        .route("/tenants/{tenant_id}", delete(delete_tenant))
}
