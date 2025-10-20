use crate::application::use_cases::create_transfer::{
    CreateTransferResponse, CreateTransferUseCase,
};
use crate::application::use_cases::get_transfer::{GetTransferResponse, GetTransferUseCase};
use crate::application::use_cases::receive_transfer::{
    ReceiveTransferResponse, ReceiveTransferUseCase,
};
use crate::application::use_cases::ship_transfer::{ShipTransferResponse, ShipTransferUseCase};
use crate::domain::entities::transfer::ReceiveTransferRequest;
use crate::infrastructure::repositories::postgres_transfer_repository::PostgresTransferRepository;
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

pub async fn create_transfer(
    State(state): State<AppState>,
    Json(request): Json<crate::domain::entities::transfer::CreateTransferRequest>,
) -> Result<Json<CreateTransferResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresTransferRepository::new(Arc::clone(&state.pool));
    let use_case = CreateTransferUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let created_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(request, created_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error creating transfer: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn get_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<GetTransferResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresTransferRepository::new(Arc::clone(&state.pool));
    let use_case = GetTransferUseCase::new(repo);

    match use_case.execute(transfer_id).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error getting transfer: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn ship_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<ShipTransferResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresTransferRepository::new(Arc::clone(&state.pool));
    let use_case = ShipTransferUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let shipped_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(transfer_id, shipped_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error shipping transfer: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn receive_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<Uuid>,
    Json(request): Json<ReceiveTransferRequest>,
) -> Result<Json<ReceiveTransferResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PostgresTransferRepository::new(Arc::clone(&state.pool));
    let use_case = ReceiveTransferUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let received_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(transfer_id, request, received_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error receiving transfer: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}
