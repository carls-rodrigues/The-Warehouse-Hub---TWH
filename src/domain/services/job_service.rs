use crate::domain::entities::job::{CreateJobRequest, Job, JobError};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait JobService: Send + Sync {
    /// Enqueue a new job for processing
    async fn enqueue_job(
        &self,
        tenant_id: Uuid,
        request: CreateJobRequest,
    ) -> Result<Job, DomainError>;

    /// Get job status by job_id
    async fn get_job_status(
        &self,
        tenant_id: Uuid,
        job_id: &str,
    ) -> Result<Option<Job>, DomainError>;

    /// Update job progress
    async fn update_job_progress(&self, job_id: &str, progress: i32) -> Result<(), DomainError>;

    /// Mark job as completed successfully
    async fn complete_job_success(
        &self,
        job_id: &str,
        result_url: Option<String>,
    ) -> Result<(), DomainError>;

    /// Mark job as failed
    async fn complete_job_failure(
        &self,
        job_id: &str,
        errors: Vec<JobError>,
    ) -> Result<(), DomainError>;

    /// Mark job as partially successful
    async fn complete_job_partial_success(
        &self,
        job_id: &str,
        result_url: Option<String>,
        errors: Vec<JobError>,
    ) -> Result<(), DomainError>;

    /// Start processing a job
    async fn start_job_processing(&self, job_id: &str) -> Result<(), DomainError>;

    /// Find jobs by status for a tenant
    async fn find_by_status(
        &self,
        tenant_id: Uuid,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Job>, DomainError>;
}
