use crate::domain::entities::job::CreateJobRequest;
use crate::domain::services::job_service::JobService;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnqueueJobRequest {
    pub tenant_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnqueueJobResponse {
    pub job_id: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct EnqueueJobUseCase<S: JobService> {
    job_service: Arc<S>,
}

impl<S: JobService> EnqueueJobUseCase<S> {
    pub fn new(job_service: Arc<S>) -> Self {
        Self { job_service }
    }

    pub async fn execute(
        &self,
        request: EnqueueJobRequest,
    ) -> Result<EnqueueJobResponse, DomainError> {
        let create_request = CreateJobRequest {
            job_type: request.job_type,
            payload: request.payload,
        };

        let job = self
            .job_service
            .enqueue_job(request.tenant_id, create_request)
            .await?;

        Ok(EnqueueJobResponse {
            job_id: job.job_id,
            status: job.status.to_string(),
            created_at: job.created_at,
        })
    }
}
