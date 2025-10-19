use crate::application::use_cases::login::LoginRequest;
use crate::shared::error::DomainError;
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

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
                message: format!("Authentication failed: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_request_dto_conversion() {
        let dto = LoginRequestDto {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        // Test that DTO can be converted to domain request
        let domain_request = crate::application::use_cases::login::LoginRequest {
            email: dto.email.clone(),
            password: dto.password.clone(),
        };

        assert_eq!(domain_request.email, "test@example.com");
        assert_eq!(domain_request.password, "password");
    }

    #[tokio::test]
    async fn test_login_response_dto_conversion() {
        let domain_response = crate::application::use_cases::login::LoginResponse {
            token: "mock-jwt-token".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            email: "test@example.com".to_string(),
            expires_at: 1234567890,
        };

        let dto = LoginResponseDto {
            token: domain_response.token.clone(),
            user_id: domain_response.user_id.clone(),
            email: domain_response.email.clone(),
            expires_at: domain_response.expires_at,
        };

        assert_eq!(dto.token, "mock-jwt-token");
        assert_eq!(dto.email, "test@example.com");
        assert_eq!(dto.user_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(dto.expires_at, 1234567890);
    }

    #[tokio::test]
    async fn test_error_response_dto_structure() {
        let error_response = ErrorResponse {
            error: "INVALID_CREDENTIALS".to_string(),
            message: "Invalid credentials".to_string(),
        };

        assert_eq!(error_response.error, "INVALID_CREDENTIALS");
        assert_eq!(error_response.message, "Invalid credentials");
    }

    #[tokio::test]
    async fn test_error_response_for_internal_error() {
        let error_response = ErrorResponse {
            error: "INTERNAL_ERROR".to_string(),
            message: "Authentication failed: Database connection failed".to_string(),
        };

        assert_eq!(error_response.error, "INTERNAL_ERROR");
        assert!(error_response.message.contains("Authentication failed"));
    }
}
