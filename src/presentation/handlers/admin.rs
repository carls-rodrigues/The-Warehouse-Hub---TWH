use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
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

#[derive(Serialize)]
pub struct TenantQuotasResponse {
    pub tenant_id: Uuid,
    pub max_items: i32,
    pub max_locations: i32,
    pub max_webhooks: i32,
    pub max_api_calls_per_hour: i32,
    pub max_storage_mb: i32,
    pub current_items: i32,
    pub current_locations: i32,
    pub current_webhooks: i32,
    pub current_storage_mb: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct UpdateTenantQuotasRequest {
    pub max_items: Option<i32>,
    pub max_locations: Option<i32>,
    pub max_webhooks: Option<i32>,
    pub max_api_calls_per_hour: Option<i32>,
    pub max_storage_mb: Option<i32>,
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

pub async fn get_tenant_quotas_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantQuotasResponse>, StatusCode> {
    let quota = sqlx::query_as!(
        TenantQuotasResponse,
        r#"
        SELECT
            tenant_id,
            max_items,
            max_locations,
            max_webhooks,
            max_api_calls_per_hour,
            max_storage_mb,
            current_items,
            current_locations,
            current_webhooks,
            current_storage_mb,
            created_at,
            updated_at
        FROM tenant_quotas
        WHERE tenant_id = $1
        "#,
        tenant_id
    )
    .fetch_optional(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match quota {
        Some(quota) => Ok(Json(quota)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_tenant_quotas_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Json(request): Json<UpdateTenantQuotasRequest>,
) -> Result<Json<TenantQuotasResponse>, StatusCode> {
    // Build dynamic update query based on provided fields
    let mut update_fields = Vec::new();
    let mut param_count = 2; // tenant_id is $1, updated_at is $2

    if request.max_items.is_some() {
        update_fields.push(format!("max_items = ${}", param_count));
        param_count += 1;
    }
    if request.max_locations.is_some() {
        update_fields.push(format!("max_locations = ${}", param_count));
        param_count += 1;
    }
    if request.max_webhooks.is_some() {
        update_fields.push(format!("max_webhooks = ${}", param_count));
        param_count += 1;
    }
    if request.max_api_calls_per_hour.is_some() {
        update_fields.push(format!("max_api_calls_per_hour = ${}", param_count));
        param_count += 1;
    }
    if request.max_storage_mb.is_some() {
        update_fields.push(format!("max_storage_mb = ${}", param_count));
        param_count += 1;
    }

    if update_fields.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let update_query = format!(
        "UPDATE tenant_quotas SET {}, updated_at = $2 WHERE tenant_id = $1 RETURNING *",
        update_fields.join(", ")
    );

    // Build parameter list
    let mut query = sqlx::query(&update_query);
    query = query.bind(tenant_id).bind(chrono::Utc::now());

    if let Some(max_items) = request.max_items {
        query = query.bind(max_items);
    }
    if let Some(max_locations) = request.max_locations {
        query = query.bind(max_locations);
    }
    if let Some(max_webhooks) = request.max_webhooks {
        query = query.bind(max_webhooks);
    }
    if let Some(max_api_calls_per_hour) = request.max_api_calls_per_hour {
        query = query.bind(max_api_calls_per_hour);
    }
    if let Some(max_storage_mb) = request.max_storage_mb {
        query = query.bind(max_storage_mb);
    }

    let row = query
        .fetch_one(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = TenantQuotasResponse {
        tenant_id: row.get("tenant_id"),
        max_items: row.get("max_items"),
        max_locations: row.get("max_locations"),
        max_webhooks: row.get("max_webhooks"),
        max_api_calls_per_hour: row.get("max_api_calls_per_hour"),
        max_storage_mb: row.get("max_storage_mb"),
        current_items: row.get("current_items"),
        current_locations: row.get("current_locations"),
        current_webhooks: row.get("current_webhooks"),
        current_storage_mb: row.get("current_storage_mb"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(response))
}
