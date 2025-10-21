use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    get_webhook_deliveries::{GetWebhookDeliveriesUseCase, GetWebhookDeliveryDetailsUseCase},
    retry_webhook_delivery::RetryWebhookDeliveryUseCase,
    test_webhook::TestWebhookUseCase,
};
use crate::shared::error::DomainError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// Get webhook deliveries
pub async fn get_webhook_deliveries(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use the user ID from login - authentication middleware will be added later
    let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let use_case = GetWebhookDeliveriesUseCase::new(state.webhook_repository.clone());

    match use_case
        .execute(webhook_id, user_id, pagination.page, pagination.limit)
        .await
    {
        Ok(response) => Ok(Json(serde_json::to_value(response).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "ValidationError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

// Get webhook delivery details
pub async fn get_webhook_delivery_details(
    State(state): State<AppState>,
    Path(delivery_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use the user ID from login - authentication middleware will be added later
    let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let use_case = GetWebhookDeliveryDetailsUseCase::new(state.webhook_repository.clone());

    match use_case.execute(delivery_id, user_id).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "ValidationError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

// Test webhook
pub async fn test_webhook(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use the user ID from login - authentication middleware will be added later
    let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let use_case = TestWebhookUseCase::new(
        state.webhook_repository.clone(),
        state.webhook_dispatcher.clone(),
    );

    match use_case.execute(webhook_id, user_id).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "ValidationError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

// Retry webhook delivery
pub async fn retry_webhook_delivery(
    State(state): State<AppState>,
    Path(delivery_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use the user ID from login - authentication middleware will be added later
    let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let use_case = RetryWebhookDeliveryUseCase::new(
        state.webhook_dispatcher.clone(),
        state.webhook_repository.clone(),
    );

    match use_case.execute(delivery_id, user_id).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "ValidationError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
