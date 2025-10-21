use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    enqueue_job::{EnqueueJobRequest, EnqueueJobUseCase},
    get_job_status::{GetJobStatusRequest, GetJobStatusUseCase},
};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct EnqueueJobPayload {
    #[serde(rename = "type")]
    pub r#type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct EnqueueJobResponse {
    pub job_id: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub status: String,
    pub progress: i32,
    pub result_url: Option<String>,
    pub errors: Option<Vec<serde_json::Value>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Enqueue a new async job
pub async fn enqueue_job(
    State(state): State<AppState>,
    Json(payload): Json<EnqueueJobPayload>,
) -> Result<(StatusCode, Json<EnqueueJobResponse>), (StatusCode, Json<ErrorResponse>)> {
    // For now, use a hardcoded tenant ID - tenant isolation will be added later
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    match state
        .enqueue_job_use_case
        .execute(EnqueueJobRequest {
            tenant_id,
            job_type: payload.r#type,
            payload: payload.payload,
        })
        .await
    {
        Ok(response) => Ok((
            StatusCode::ACCEPTED,
            Json(EnqueueJobResponse {
                job_id: response.job_id,
                status: response.status,
                created_at: response.created_at,
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Bad Request".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

/// Get job status by job ID
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // For now, use a hardcoded tenant ID - tenant isolation will be added later
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    match state
        .get_job_status_use_case
        .execute(GetJobStatusRequest { tenant_id, job_id })
        .await
    {
        Ok(Some(response)) => {
            let errors = response.errors.map(|e| {
                e.into_iter()
                    .map(|error| serde_json::to_value(error).unwrap_or_default())
                    .collect()
            });

            Ok(Json(JobStatusResponse {
                job_id: response.job_id,
                r#type: response.job_type,
                status: response.status,
                progress: response.progress,
                result_url: response.result_url,
                errors,
                created_at: response.created_at,
                updated_at: response.updated_at,
                started_at: response.started_at,
                completed_at: response.completed_at,
            }))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Not Found".to_string(),
                message: "Job not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal Server Error".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
