use crate::application::use_cases::login::{LoginRequest, LoginUseCase};
use crate::infrastructure::repositories::postgres_user_repository::PostgresUserRepository;
use crate::shared::error::DomainError;
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LoginRequestDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponseDto {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(request): Json<LoginRequestDto>,
) -> Result<Json<LoginResponseDto>, (StatusCode, Json<ErrorResponse>)> {
    // Convert DTO to domain request
    let login_request = LoginRequest {
        email: request.email,
        password: request.password,
    };

    // Execute the use case
    match state.login_use_case.execute(login_request).await {
        Ok(response) => {
            let dto = LoginResponseDto {
                token: response.token,
                user_id: response.user_id,
                email: response.email,
                expires_at: response.expires_at,
            };
            Ok(Json(dto))
        }
        Err(DomainError::ValidationError(msg)) => {
            let error_response = ErrorResponse {
                error: "INVALID_CREDENTIALS".to_string(),
                message: msg,
            };
            Err((StatusCode::UNAUTHORIZED, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Authentication failed: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
