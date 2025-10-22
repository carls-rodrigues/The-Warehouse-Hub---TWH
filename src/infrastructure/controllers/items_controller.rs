use crate::application::use_cases::{
    create_item::{CreateItemRequest, CreateItemUseCase},
    delete_item::{DeleteItemRequest, DeleteItemUseCase},
    get_item::{GetItemRequest, GetItemUseCase},
    list_items::{ListItemsRequest, ListItemsUseCase},
    update_item::{UpdateItemRequest, UpdateItemUseCase},
};
use crate::infrastructure::repositories::postgres_item_repository::PostgresItemRepository;
use crate::shared::error::DomainError;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// DTOs for API requests/responses

#[derive(Debug, Deserialize)]
pub struct CreateItemRequestDto {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: String,
    pub barcode: Option<String>,
    pub cost_price: f64,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CreateItemResponseDto {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub cost_price: f64,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct GetItemResponseDto {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: String,
    pub barcode: Option<String>,
    pub cost_price: f64,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemRequestDto {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub barcode: Option<String>,
    pub cost_price: Option<f64>,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct UpdateItemResponseDto {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub unit: String,
    pub cost_price: f64,
    pub active: bool,
    pub updated_at: String,
    pub etag: String,
}

#[derive(Debug, Serialize)]
pub struct ItemSummaryDto {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: String,
    pub cost_price: f64,
    pub sale_price: Option<f64>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListItemsResponseDto {
    pub items: Vec<ItemSummaryDto>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct DeleteItemResponseDto {
    pub id: String,
    pub active: bool,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// Query parameters for list endpoint
#[derive(Debug, Deserialize)]
pub struct ListItemsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Handler functions

pub async fn create_item_handler(
    State(state): State<AppState>,
    tenant_context: Option<
        Extension<crate::infrastructure::middleware::tenant_middleware::TenantContext>,
    >,
    Json(request): Json<CreateItemRequestDto>,
) -> Result<(StatusCode, Json<CreateItemResponseDto>), (StatusCode, Json<ErrorResponse>)> {
    // Extract tenant_id from extension or default to sandbox tenant
    let tenant_id = tenant_context
        .map(|ext| ext.tenant_id)
        .unwrap_or_else(|| uuid::Uuid::parse_str("d60a7de9-1009-4606-aae9-ae6ffe5827aa").unwrap());

    // Set tenant context on the connection pool
    sqlx::query!("SELECT set_tenant_context($1)", tenant_id)
        .execute(&*state.pool)
        .await
        .map_err(|e| {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to set tenant context: {e}"),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    // Initialize use case
    let item_repository = Arc::new(PostgresItemRepository::new(Arc::clone(&state.pool)));
    let use_case = CreateItemUseCase::new(item_repository); // Convert DTO to domain request
    let domain_request = CreateItemRequest {
        sku: request.sku,
        name: request.name,
        description: request.description,
        category: request.category,
        unit: request.unit,
        barcode: request.barcode,
        cost_price: request.cost_price,
        sale_price: request.sale_price,
        reorder_point: request.reorder_point,
        reorder_qty: request.reorder_qty,
        weight: request.weight,
        dimensions: request
            .dimensions
            .and_then(|d| serde_json::from_value(d).ok()),
        metadata: request.metadata,
    };

    // Execute use case
    match use_case.execute(domain_request).await {
        Ok(response) => {
            let dto = CreateItemResponseDto {
                id: response.id.to_string(),
                sku: response.sku,
                name: response.name,
                unit: response.unit,
                cost_price: response.cost_price,
                active: response.active,
                created_at: response.created_at.to_rfc3339(),
                updated_at: response.updated_at.to_rfc3339(),
            };
            Ok((StatusCode::CREATED, Json(dto)))
        }
        Err(DomainError::ValidationError(msg)) => {
            let error_response = ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: msg,
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to create item: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_item_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GetItemResponseDto>, (StatusCode, Json<ErrorResponse>)> {
    // Parse UUID
    let item_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "INVALID_ID".to_string(),
                message: "Invalid item ID format".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Initialize use case
    let item_repository = Arc::new(PostgresItemRepository::new(Arc::clone(&state.pool)));
    let use_case = GetItemUseCase::new(item_repository);

    // Execute use case
    match use_case.execute(GetItemRequest { id: item_id }).await {
        Ok(response) => {
            let dto = GetItemResponseDto {
                id: response.id.to_string(),
                sku: response.sku,
                name: response.name,
                description: response.description,
                category: response.category,
                unit: response.unit,
                barcode: response.barcode,
                cost_price: response.cost_price,
                sale_price: response.sale_price,
                reorder_point: response.reorder_point,
                reorder_qty: response.reorder_qty,
                weight: response.weight,
                dimensions: response.dimensions,
                metadata: response.metadata,
                active: response.active,
                created_at: response.created_at.to_rfc3339(),
                updated_at: response.updated_at.to_rfc3339(),
            };
            Ok(Json(dto))
        }
        Err(DomainError::ValidationError(msg)) if msg.contains("not found") => {
            let error_response = ErrorResponse {
                error: "ITEM_NOT_FOUND".to_string(),
                message: msg,
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(DomainError::ValidationError(msg)) => {
            let error_response = ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: msg,
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to get item: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn update_item_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<UpdateItemRequestDto>,
) -> Result<(StatusCode, Json<UpdateItemResponseDto>), (StatusCode, Json<ErrorResponse>)> {
    // Parse UUID
    let item_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "INVALID_ID".to_string(),
                message: "Invalid item ID format".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Get If-Match header for optimistic concurrency
    let if_match_etag = headers
        .get("if-match")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Initialize use case
    let item_repository = Arc::new(PostgresItemRepository::new(Arc::clone(&state.pool)));
    let use_case = UpdateItemUseCase::new(item_repository);

    // Convert DTO to domain request
    let domain_request = UpdateItemRequest {
        id: item_id,
        sku: request.sku,
        name: request.name,
        description: request.description,
        category: request.category,
        unit: request.unit,
        barcode: request.barcode,
        cost_price: request.cost_price,
        sale_price: request.sale_price,
        reorder_point: request.reorder_point,
        reorder_qty: request.reorder_qty,
        weight: request.weight,
        dimensions: request.dimensions,
        metadata: request.metadata,
        if_match: if_match_etag,
    };

    // Execute use case
    match use_case.execute(domain_request).await {
        Ok(response) => {
            let dto = UpdateItemResponseDto {
                id: response.id.to_string(),
                sku: response.sku,
                name: response.name,
                unit: response.unit,
                cost_price: response.cost_price,
                active: response.active,
                updated_at: response.updated_at.to_rfc3339(),
                etag: response.etag,
            };
            Ok((StatusCode::OK, Json(dto)))
        }
        Err(DomainError::ValidationError(msg)) if msg.contains("not found") => {
            let error_response = ErrorResponse {
                error: "ITEM_NOT_FOUND".to_string(),
                message: msg,
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(DomainError::ValidationError(msg))
            if msg.contains("ETag") || msg.contains("concurrent") =>
        {
            let error_response = ErrorResponse {
                error: "CONCURRENT_MODIFICATION".to_string(),
                message: msg,
            };
            Err((StatusCode::PRECONDITION_FAILED, Json(error_response)))
        }
        Err(DomainError::ValidationError(msg)) => {
            let error_response = ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: msg,
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to update item: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn list_items_handler(
    State(state): State<AppState>,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<ListItemsResponseDto>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize use case
    let item_repository = Arc::new(PostgresItemRepository::new(Arc::clone(&state.pool)));
    let use_case = ListItemsUseCase::new(item_repository);

    // Execute use case
    match use_case
        .execute(ListItemsRequest {
            limit: query.limit,
            offset: query.offset,
        })
        .await
    {
        Ok(response) => {
            let items_dto = response
                .items
                .into_iter()
                .map(|item| ItemSummaryDto {
                    id: item.id.to_string(),
                    sku: item.sku,
                    name: item.name,
                    category: item.category,
                    unit: item.unit,
                    cost_price: item.cost_price,
                    sale_price: item.sale_price,
                    active: item.active,
                    created_at: item.created_at.to_rfc3339(),
                    updated_at: item.updated_at.to_rfc3339(),
                })
                .collect();

            let dto = ListItemsResponseDto {
                items: items_dto,
                total_count: response.total_count,
                limit: response.limit,
                offset: response.offset,
            };
            Ok(Json(dto))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to list items: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn delete_item_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteItemResponseDto>, (StatusCode, Json<ErrorResponse>)> {
    // Parse UUID
    let item_id = match Uuid::parse_str(&id) {
        Ok(uuid) => uuid,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "INVALID_ID".to_string(),
                message: "Invalid item ID format".to_string(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Initialize use case
    let item_repository = Arc::new(PostgresItemRepository::new(Arc::clone(&state.pool)));
    let use_case = DeleteItemUseCase::new(item_repository);

    // Execute use case
    match use_case.execute(DeleteItemRequest { id: item_id }).await {
        Ok(response) => {
            let dto = DeleteItemResponseDto {
                id: response.id.to_string(),
                active: response.active,
                updated_at: response.updated_at.to_rfc3339(),
            };
            Ok(Json(dto))
        }
        Err(DomainError::ValidationError(msg)) if msg.contains("not found") => {
            let error_response = ErrorResponse {
                error: "ITEM_NOT_FOUND".to_string(),
                message: msg,
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(DomainError::ValidationError(msg)) if msg.contains("already deleted") => {
            let error_response = ErrorResponse {
                error: "ITEM_ALREADY_DELETED".to_string(),
                message: msg,
            };
            Err((StatusCode::CONFLICT, Json(error_response)))
        }
        Err(DomainError::ValidationError(msg)) => {
            let error_response = ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: msg,
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to delete item: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
