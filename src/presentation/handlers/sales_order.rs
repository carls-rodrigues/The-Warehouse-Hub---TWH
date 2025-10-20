use crate::application::use_cases::{
    create_sales_order::{
        CreateSalesOrderRequest, CreateSalesOrderResponse, CreateSalesOrderUseCase,
    },
    get_sales_order::{GetSalesOrderUseCase, SalesOrderWithLines},
    ship_sales_order::{ShipSalesOrderRequest, ShipSalesOrderResponse, ShipSalesOrderUseCase},
};
use crate::infrastructure::repositories::postgres_sales_order_repository::PostgresSalesOrderRepository;
use crate::shared::error::DomainError;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_sales_order(
    State(state): State<AppState>,
    Json(request): Json<CreateSalesOrderRequest>,
) -> Result<Json<CreateSalesOrderResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresSalesOrderRepository::new(Arc::clone(&state.pool));
    let use_case = CreateSalesOrderUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let created_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(request, created_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error creating sales order: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn get_sales_order(
    State(state): State<AppState>,
    Path(so_id): Path<Uuid>,
) -> Result<Json<SalesOrderWithLines>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresSalesOrderRepository::new(Arc::clone(&state.pool));
    let use_case = GetSalesOrderUseCase::new(repo);

    match use_case.execute(so_id).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error getting sales order: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn ship_sales_order(
    State(state): State<AppState>,
    Path(so_id): Path<Uuid>,
    Json(request): Json<ShipSalesOrderRequest>,
) -> Result<Json<ShipSalesOrderResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresSalesOrderRepository::new(Arc::clone(&state.pool));
    let use_case = ShipSalesOrderUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let created_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(so_id, request, created_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) | Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error shipping sales order: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}
