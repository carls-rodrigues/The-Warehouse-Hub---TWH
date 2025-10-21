use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

#[derive(Serialize)]
pub struct AdminDashboardResponse {
    pub total_tenants: i64,
    pub active_sandboxes: i64,
    pub expired_sandboxes: i64,
    pub total_webhook_deliveries: i64,
    pub failed_webhook_deliveries: i64,
}

#[derive(Serialize)]
pub struct SandboxTenant {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct ListSandboxesResponse {
    pub sandboxes: Vec<SandboxTenant>,
}

pub async fn admin_dashboard_handler(
    State(state): State<AppState>,
) -> Result<Json<AdminDashboardResponse>, StatusCode> {
    let tenants = state
        .list_tenants_use_case
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_sandboxes = tenants
        .iter()
        .filter(|t| t.tenant_type.as_str() == "SANDBOX" && t.status.as_str() == "ACTIVE")
        .count() as i64;

    let expired_sandboxes = tenants
        .iter()
        .filter(|t| t.tenant_type.as_str() == "SANDBOX" && t.status.as_str() == "EXPIRED")
        .count() as i64;

    // Get actual webhook delivery counts
    let total_webhook_deliveries = sqlx::query!("SELECT COUNT(*) as count FROM webhook_deliveries")
        .fetch_one(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .count
        .unwrap_or(0);

    let failed_webhook_deliveries = sqlx::query!(
        "SELECT COUNT(*) as count FROM webhook_deliveries WHERE status IN ('FAILED', 'TIMEOUT', 'DLQ')"
    )
    .fetch_one(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .count
    .unwrap_or(0);

    Ok(Json(AdminDashboardResponse {
        total_tenants: tenants.len() as i64,
        active_sandboxes,
        expired_sandboxes,
        total_webhook_deliveries,
        failed_webhook_deliveries,
    }))
}

pub async fn list_sandboxes_handler(
    State(state): State<AppState>,
) -> Result<Json<ListSandboxesResponse>, StatusCode> {
    let tenants = state
        .list_tenants_use_case
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sandboxes = tenants
        .into_iter()
        .filter(|t| t.tenant_type.as_str() == "SANDBOX")
        .map(|t| SandboxTenant {
            id: t.id,
            name: t.name,
            status: t.status.as_str().to_string(),
            created_at: t.created_at,
            expires_at: t.expires_at,
        })
        .collect();

    Ok(Json(ListSandboxesResponse { sandboxes }))
}

pub async fn cleanup_expired_sandboxes_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cleaned_ids = state
        .cleanup_expired_sandboxes_use_case
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "message": format!("Cleaned up {} expired sandboxes", cleaned_ids.len()),
        "cleaned_tenant_ids": cleaned_ids
    })))
}

#[derive(Serialize)]
pub struct DlqDeliveryResponse {
    pub deliveries: Vec<serde_json::Value>,
    pub pagination: serde_json::Value,
}

pub async fn list_dlq_deliveries_handler(
    State(state): State<AppState>,
) -> Result<Json<DlqDeliveryResponse>, StatusCode> {
    let result = state
        .list_dlq_deliveries_use_case
        .execute(None, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert deliveries to JSON values for serialization
    let deliveries = result
        .deliveries
        .into_iter()
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "webhook_id": d.webhook_id,
                "event_id": d.event_id,
                "status": d.status.as_str(),
                "attempt_count": d.attempt_count,
                "last_attempt_at": d.last_attempt_at,
                "response_status": d.response_status,
                "error_message": d.error_message,
                "created_at": d.created_at
            })
        })
        .collect();

    let pagination = serde_json::json!({
        "page": result.pagination.page,
        "per_page": result.pagination.per_page,
        "total": result.pagination.total,
        "total_pages": result.pagination.total_pages
    });

    Ok(Json(DlqDeliveryResponse {
        deliveries,
        pagination,
    }))
}

#[derive(Deserialize)]
pub struct ReplayDlqRequest {
    pub delivery_id: Uuid,
}

pub async fn replay_dlq_delivery_handler(
    State(state): State<AppState>,
    Json(request): Json<ReplayDlqRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = state
        .replay_dlq_delivery_use_case
        .execute(request.delivery_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "success": result.success,
        "message": result.message,
        "new_status": result.new_status
    })))
}

pub async fn get_billing_metrics_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = state
        .get_billing_metrics_use_case
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "total_api_calls": metrics.total_api_calls,
        "storage_used_gb": metrics.storage_used_gb,
        "active_tenants": metrics.active_tenants,
        "total_items": metrics.total_items,
        "total_locations": metrics.total_locations,
        "total_orders": metrics.total_orders,
        "total_transfers": metrics.total_transfers,
        "webhook_deliveries": {
            "total": metrics.webhook_deliveries.total,
            "successful": metrics.webhook_deliveries.successful,
            "failed": metrics.webhook_deliveries.failed
        },
        "billing_period": {
            "start_date": metrics.billing_period.start_date,
            "end_date": metrics.billing_period.end_date,
            "days_remaining": metrics.billing_period.days_remaining
        }
    })))
}
