use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    create_purchase_order::{CreatePurchaseOrderUseCase, CreatePurchaseOrderUseCaseRequest, CreatePurchaseOrderResponse},
    get_purchase_order::{GetPurchaseOrderUseCase, GetPurchaseOrderResponse},
    receive_purchase_order::{ReceivePurchaseOrderUseCase, ReceivePurchaseOrderUseCaseRequest, ReceivePurchaseOrderResponse},
};
use crate::domain::entities::purchase_order::{CreatePurchaseOrderLine, ReceiveLine};
use crate::shared::error::DomainError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseOrderRequest {
    pub supplier_id: Uuid,
    pub expected_date: Option<chrono::DateTime<chrono::Utc>>,
    pub lines: Vec<CreatePurchaseOrderLine>,
}

#[derive(Debug, Deserialize)]
pub struct ReceivePurchaseOrderRequest {
    pub received_lines: Vec<ReceiveLine>,
    pub receive_date: Option<chrono::DateTime<chrono::Utc>>,
    pub destination_location_id: Uuid,
}

/// Create a new purchase order
pub async fn create_purchase_order(
    State(state): State<AppState>,
    Json(request): Json<CreatePurchaseOrderRequest>,
) -> Result<(StatusCode, Json<CreatePurchaseOrderResponse>), (StatusCode, Json<ErrorResponse>)> {
    let use_case_request = CreatePurchaseOrderUseCaseRequest {
        supplier_id: request.supplier_id,
        expected_date: request.expected_date,
        lines: request.lines,
    };

    // TODO: Get user ID from authentication context
    let created_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match state
        .create_purchase_order_use_case
        .execute(use_case_request, created_by)
        .await
    {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "PurchaseOrderError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

/// Get a purchase order by ID
pub async fn get_purchase_order(
    State(state): State<AppState>,
    Path(po_id): Path<Uuid>,
) -> Result<Json<GetPurchaseOrderResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .get_purchase_order_use_case
        .execute(po_id)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "NotFound".to_string(),
                        message: e.to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "PurchaseOrderError".to_string(),
                        message: e.to_string(),
                    }),
                ))
            }
        }
    }
}

/// Receive items for a purchase order
pub async fn receive_purchase_order(
    State(state): State<AppState>,
    Path(po_id): Path<Uuid>,
    Json(request): Json<ReceivePurchaseOrderRequest>,
) -> Result<Json<ReceivePurchaseOrderResponse>, (StatusCode, Json<ErrorResponse>)> {
    let use_case_request = ReceivePurchaseOrderUseCaseRequest {
        po_id,
        received_lines: request.received_lines,
        receive_date: request.receive_date,
        destination_location_id: request.destination_location_id,
    };

    // TODO: Get user ID from authentication context
    let received_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match state
        .receive_purchase_order_use_case
        .execute(use_case_request, received_by)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "ReceiveError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}