use crate::domain::entities::job::{Job, JobError};
use crate::domain::services::job_service::JobService;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetJobStatusRequest {
    pub tenant_id: Uuid,
    pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetJobStatusResponse {
    pub job_id: String,
    pub job_type: String,
    pub status: String,
    pub progress: i32,
    pub result_url: Option<String>,
    pub errors: Option<Vec<JobError>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct GetJobStatusUseCase<S: JobService> {
    job_service: Arc<S>,
}

impl<S: JobService> GetJobStatusUseCase<S> {
    pub fn new(job_service: Arc<S>) -> Self {
        Self { job_service }
    }

    pub async fn execute(
        &self,
        request: GetJobStatusRequest,
    ) -> Result<Option<GetJobStatusResponse>, DomainError> {
        let job = self
            .job_service
            .get_job_status(request.tenant_id, &request.job_id)
            .await?;

        match job {
            Some(job) => Ok(Some(GetJobStatusResponse {
                job_id: job.job_id,
                job_type: job.job_type,
                status: job.status.to_string(),
                progress: job.progress,
                result_url: job.result_url,
                errors: job.errors,
                created_at: job.created_at,
                updated_at: job.updated_at,
                started_at: job.started_at,
                completed_at: job.completed_at,
            })),
            None => Ok(None),
        }
    }
}
