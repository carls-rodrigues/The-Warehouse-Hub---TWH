use crate::application::use_cases::{
    create_location::{CreateLocationRequest, CreateLocationUseCase},
    delete_location::{DeleteLocationRequest, DeleteLocationUseCase},
    get_location::{GetLocationRequest, GetLocationUseCase},
    list_locations::{ListLocationsRequest, ListLocationsUseCase},
    update_location::{UpdateLocationRequestDto, UpdateLocationUseCase},
};
use crate::infrastructure::repositories::postgres_location_repository::PostgresLocationRepository;
use crate::shared::error::DomainError;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// DTOs for API requests/responses

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequestDto {
    pub name: String,
    pub code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateLocationResponseDto {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct GetLocationResponseDto {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub r#type: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLocationRequestDtoApi {
    pub name: Option<String>,
    pub code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateLocationResponseDto {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub r#type: Option<String>,
    pub active: bool,
    pub updated_at: String,
    pub etag: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteLocationResponseDto {
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
pub struct ListLocationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Handler functions

pub async fn create_location_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateLocationRequestDto>,
) -> Result<(StatusCode, Json<CreateLocationResponseDto>), (StatusCode, Json<ErrorResponse>)> {
    // Initialize use case
    let location_repository = Arc::new(PostgresLocationRepository::new(Arc::clone(&state.pool)));
    let use_case = CreateLocationUseCase::new(location_repository);

    // Convert DTO to domain request
    let domain_request = CreateLocationRequest {
        name: request.name,
        code: request.code,
        address: request.address.and_then(|a| serde_json::from_value(a).ok()),
        r#type: request.r#type,
    };

    // Execute use case
    match use_case.execute(domain_request).await {
        Ok(response) => {
            let dto = CreateLocationResponseDto {
                id: response.id.to_string(),
                name: response.name,
                code: response.code,
                r#type: response.r#type,
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
                message: format!("Failed to create location: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn get_location_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GetLocationResponseDto>, (StatusCode, Json<ErrorResponse>)> {
    // Parse UUID
    let location_id = Uuid::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_ID".to_string(),
                message: "Invalid location ID format".to_string(),
            }),
        )
    })?;

    // Initialize use case
    let location_repository = Arc::new(PostgresLocationRepository::new(Arc::clone(&state.pool)));
    let use_case = GetLocationUseCase::new(location_repository);

    // Execute use case
    match use_case
        .execute(GetLocationRequest { id: location_id })
        .await
    {
        Ok(response) => {
            let dto = GetLocationResponseDto {
                id: response.id.to_string(),
                name: response.name,
                code: response.code,
                address: response
                    .address
                    .map(|a| serde_json::to_value(a).unwrap_or_default()),
                r#type: response.r#type,
                active: response.active,
                created_at: response.created_at.to_rfc3339(),
                updated_at: response.updated_at.to_rfc3339(),
            };
            Ok(Json(dto))
        }
        Err(DomainError::NotFound(msg)) => {
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: msg,
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to get location: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn update_location_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateLocationRequestDtoApi>,
) -> Result<(StatusCode, Json<UpdateLocationResponseDto>), (StatusCode, Json<ErrorResponse>)> {
    // Parse UUID
    let location_id = Uuid::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_ID".to_string(),
                message: "Invalid location ID format".to_string(),
            }),
        )
    })?;

    // Initialize use case
    let location_repository = Arc::new(PostgresLocationRepository::new(Arc::clone(&state.pool)));
    let use_case = UpdateLocationUseCase::new(location_repository);

    // Convert DTO to domain request
    let domain_request = UpdateLocationRequestDto {
        name: request.name,
        code: request.code,
        address: request.address.and_then(|a| serde_json::from_value(a).ok()),
        r#type: request.r#type,
    };

    // Execute use case
    match use_case.execute(location_id, domain_request).await {
        Ok(response) => {
            let dto = UpdateLocationResponseDto {
                id: response.id.to_string(),
                name: response.name,
                code: response.code,
                r#type: response.r#type,
                active: response.active,
                updated_at: response.updated_at.to_rfc3339(),
                etag: response.etag,
            };
            Ok((StatusCode::OK, Json(dto)))
        }
        Err(DomainError::ValidationError(msg)) => {
            let error_response = ErrorResponse {
                error: "VALIDATION_ERROR".to_string(),
                message: msg,
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
        Err(DomainError::NotFound(msg)) => {
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: msg,
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to update location: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn delete_location_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteLocationResponseDto>, (StatusCode, Json<ErrorResponse>)> {
    // Parse UUID
    let location_id = Uuid::parse_str(&id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_ID".to_string(),
                message: "Invalid location ID format".to_string(),
            }),
        )
    })?;

    // Initialize use case
    let location_repository = Arc::new(PostgresLocationRepository::new(Arc::clone(&state.pool)));
    let use_case = DeleteLocationUseCase::new(location_repository);

    // Execute use case
    match use_case
        .execute(DeleteLocationRequest { id: location_id })
        .await
    {
        Ok(response) => {
            let dto = DeleteLocationResponseDto {
                id: response.id.to_string(),
                active: response.active,
                updated_at: response.updated_at.to_rfc3339(),
            };
            Ok(Json(dto))
        }
        Err(DomainError::NotFound(msg)) => {
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: msg,
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to delete location: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn list_locations_handler(
    State(state): State<AppState>,
    Query(query): Query<ListLocationsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize use case
    let location_repository = Arc::new(PostgresLocationRepository::new(Arc::clone(&state.pool)));
    let use_case = ListLocationsUseCase::new(location_repository);

    // Execute use case
    match use_case
        .execute(ListLocationsRequest {
            limit: query.limit,
            offset: query.offset,
        })
        .await
    {
        Ok(response) => {
            // Convert to the expected API format
            let api_response = serde_json::json!({
                "data": response.locations,
                "meta": {
                    "page": (response.offset / response.limit) + 1,
                    "per_page": response.limit,
                    "total": response.total_count,
                    "total_pages": (response.total_count + response.limit - 1) / response.limit
                }
            });
            Ok(Json(api_response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: "INTERNAL_ERROR".to_string(),
                message: format!("Failed to list locations: {e}"),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
