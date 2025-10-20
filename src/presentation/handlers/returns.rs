use crate::application::use_cases::create_return::{
    CreateReturnResponse, CreateReturnUseCase,
};
use crate::application::use_cases::get_return::{GetReturnResponse, GetReturnUseCase};
use crate::application::use_cases::process_return::{
    ProcessReturnResponse, ProcessReturnUseCase,
};
use crate::domain::entities::returns::ProcessReturnRequest;
use crate::infrastructure::repositories::postgres_return_repository::PostgresReturnRepository;
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

pub async fn create_return(
    State(state): State<AppState>,
    Json(request): Json<crate::domain::entities::returns::CreateReturnRequest>,
) -> Result<Json<CreateReturnResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = Arc::new(PostgresReturnRepository::new(Arc::clone(&state.pool)));
    let use_case = CreateReturnUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let created_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(request, created_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error creating return: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn get_return(
    State(state): State<AppState>,
    Path(return_id): Path<Uuid>,
) -> Result<Json<GetReturnResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = Arc::new(PostgresReturnRepository::new(Arc::clone(&state.pool)));
    let use_case = GetReturnUseCase::new(repo);

    match use_case.execute(return_id).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error getting return: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}

pub async fn process_return(
    State(state): State<AppState>,
    Path(return_id): Path<Uuid>,
    Json(request): Json<ProcessReturnRequest>,
) -> Result<Json<ProcessReturnResponse>, (StatusCode, Json<serde_json::Value>)> {
    let repo = Arc::new(PostgresReturnRepository::new(Arc::clone(&state.pool)));
    let use_case = ProcessReturnUseCase::new(repo);

    // TODO: Get user ID from authentication context
    let processed_by = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(); // Use existing test user

    match use_case.execute(return_id, request, processed_by).await {
        Ok(response) => Ok(Json(response)),
        Err(DomainError::ValidationError(msg)) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))))
        }
        Err(DomainError::NotFound(msg)) => {
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": msg }))))
        }
        Err(e) => {
            eprintln!("Error processing return: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ))
        }
    }
}