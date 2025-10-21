use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    delete_webhook::DeleteWebhookUseCase,
    register_webhook::{RegisterWebhookRequest, RegisterWebhookUseCase},
    update_webhook::{UpdateWebhookRequest, UpdateWebhookUseCase},
};
use crate::domain::services::webhook_repository::WebhookRepository;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Register a new webhook
pub async fn register_webhook(
    State(state): State<AppState>,
    Json(request): Json<RegisterWebhookRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use the user ID from login - authentication middleware will be added later
    let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let use_case = RegisterWebhookUseCase::new(state.webhook_repository.clone());

    match use_case.execute(request, user_id).await {
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

/// Update an existing webhook
pub async fn update_webhook(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
    Json(request): Json<UpdateWebhookRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use a hardcoded user ID - authentication will be added later
    let user_id = Uuid::new_v4();

    let use_case = UpdateWebhookUseCase::new(state.webhook_repository.clone());

    match use_case.execute(webhook_id, request, user_id).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => {
            let status_code = match e {
                crate::shared::error::DomainError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            };
            Err((
                status_code,
                Json(ErrorResponse {
                    error: "ValidationError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

/// Delete a webhook
pub async fn delete_webhook(
    State(state): State<AppState>,
    Path(webhook_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // For now, use a hardcoded user ID - authentication will be added later
    let user_id = Uuid::new_v4();

    let use_case = DeleteWebhookUseCase::new(state.webhook_repository.clone());

    match use_case.execute(webhook_id, user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let status_code = match e {
                crate::shared::error::DomainError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            };
            Err((
                status_code,
                Json(ErrorResponse {
                    error: "ValidationError".to_string(),
                    message: e.to_string(),
                }),
            ))
        }
    }
}

/// Get user's webhooks
pub async fn get_user_webhooks(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use a hardcoded user ID - authentication will be added later
    let user_id = Uuid::new_v4();
    match state.webhook_repository.get_user_webhooks(user_id).await {
        Ok(webhooks) => Ok(Json(serde_json::to_value(webhooks).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "DatabaseError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
