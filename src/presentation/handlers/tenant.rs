use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    cleanup_expired_sandboxes::CleanupExpiredSandboxesUseCase,
    create_sandbox_tenant::CreateSandboxTenantUseCase, create_tenant::CreateTenantUseCase,
    delete_tenant::DeleteTenantUseCase, get_tenant::GetTenantUseCase,
    list_tenants::ListTenantsUseCase,
};
use crate::domain::entities::tenant::{CreateSandboxTenantResponse, Tenant, TenantType};
use crate::shared::error::DomainError;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub tenant_type: String,
}

#[derive(Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub name: String,
    pub tenant_type: String,
    pub status: String,
    pub database_schema: String,
    pub created_by: Option<Uuid>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Tenant> for TenantResponse {
    fn from(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            name: tenant.name,
            tenant_type: tenant.tenant_type.as_str().to_string(),
            status: tenant.status.as_str().to_string(),
            database_schema: tenant.database_schema,
            created_by: tenant.created_by,
            expires_at: tenant.expires_at.map(|dt| dt.to_rfc3339()),
            created_at: tenant.created_at.to_rfc3339(),
            updated_at: tenant.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
pub struct CleanupResponse {
    pub cleaned_tenant_ids: Vec<Uuid>,
    pub count: usize,
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<Json<TenantResponse>, (StatusCode, String)> {
    // Parse tenant type
    let tenant_type = match request.tenant_type.as_str() {
        "SANDBOX" => TenantType::Sandbox,
        "PRODUCTION" => TenantType::Production,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid tenant type. Must be 'SANDBOX' or 'PRODUCTION'".to_string(),
            ))
        }
    };

    // TODO: Get user ID from authentication context
    // For now, use None (system-created tenant)
    let created_by = None; // Placeholder - should come from auth

    match state
        .create_tenant_use_case
        .execute(request.name, tenant_type, created_by)
        .await
    {
        Ok(tenant) => Ok(Json(tenant.into())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create tenant: {}", e),
        )),
    }
}

pub async fn create_sandbox_tenant(
    State(state): State<AppState>,
) -> Result<Json<CreateSandboxTenantResponse>, (StatusCode, String)> {
    // TODO: Get user ID from authentication context
    // For now, use None (system-created tenant)
    let created_by = None; // Placeholder - should come from auth

    match state
        .create_sandbox_tenant_use_case
        .execute(created_by)
        .await
    {
        Ok(tenant) => {
            let response = CreateSandboxTenantResponse {
                tenant_id: tenant.id,
                status: tenant.status.as_str().to_string(),
                expires_at: tenant.expires_at.unwrap_or_default(),
                message: "Sandbox tenant created successfully with sample data".to_string(),
            };
            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create sandbox tenant: {}", e),
        )),
    }
}

pub async fn get_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantResponse>, (StatusCode, String)> {
    match state.get_tenant_use_case.execute(tenant_id).await {
        Ok(Some(tenant)) => Ok(Json(tenant.into())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Tenant {} not found", tenant_id),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get tenant: {}", e),
        )),
    }
}

pub async fn list_tenants(
    State(state): State<AppState>,
) -> Result<Json<Vec<TenantResponse>>, (StatusCode, String)> {
    match state.list_tenants_use_case.execute().await {
        Ok(tenants) => {
            let responses = tenants.into_iter().map(TenantResponse::from).collect();
            Ok(Json(responses))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list tenants: {}", e),
        )),
    }
}

pub async fn delete_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.delete_tenant_use_case.execute(tenant_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete tenant: {}", e),
        )),
    }
}

pub async fn cleanup_expired_sandboxes(
    State(state): State<AppState>,
) -> Result<Json<CleanupResponse>, (StatusCode, String)> {
    match state.cleanup_expired_sandboxes_use_case.execute().await {
        Ok(cleaned_ids) => {
            let count = cleaned_ids.len();
            Ok(Json(CleanupResponse {
                cleaned_tenant_ids: cleaned_ids,
                count,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to cleanup expired sandboxes: {}", e),
        )),
    }
}
