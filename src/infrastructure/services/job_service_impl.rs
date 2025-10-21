use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    entities::job::{CreateJobRequest, Job, JobError},
    services::{job_repository::JobRepository, job_service::JobService},
};

pub struct JobServiceImpl<T: JobRepository> {
    job_repository: Arc<T>,
}

impl<T: JobRepository> JobServiceImpl<T> {
    pub fn new(job_repository: Arc<T>) -> Self {
        Self { job_repository }
    }
}

#[async_trait]
impl<T: JobRepository> JobService for JobServiceImpl<T> {
    async fn enqueue_job(
        &self,
        tenant_id: Uuid,
        request: CreateJobRequest,
    ) -> Result<Job, crate::shared::error::DomainError> {
        // Create new job entity
        let mut job = Job::new(tenant_id, request.job_type, Some(request.payload))?;

        // Save to repository
        self.job_repository.save(&job).await?;

        Ok(job)
    }

    async fn get_job_status(
        &self,
        tenant_id: Uuid,
        job_id: &str,
    ) -> Result<Option<Job>, crate::shared::error::DomainError> {
        let job = self.job_repository.find_by_job_id(job_id).await?;

        // Ensure job belongs to the requesting tenant
        if let Some(ref job) = job {
            if job.tenant_id != tenant_id {
                return Err(crate::shared::error::DomainError::ValidationError(
                    "Job not found".to_string(),
                ));
            }
        }

        Ok(job)
    }

    async fn update_job_progress(
        &self,
        job_id: &str,
        progress: i32,
    ) -> Result<(), crate::shared::error::DomainError> {
        let mut job = self
            .job_repository
            .find_by_job_id(job_id)
            .await?
            .ok_or_else(|| {
                crate::shared::error::DomainError::ValidationError("Job not found".to_string())
            })?;

        job.update_progress(progress);
        self.job_repository.update(&job).await?;

        Ok(())
    }

    async fn complete_job_success(
        &self,
        job_id: &str,
        result_url: Option<String>,
    ) -> Result<(), crate::shared::error::DomainError> {
        let mut job = self
            .job_repository
            .find_by_job_id(job_id)
            .await?
            .ok_or_else(|| {
                crate::shared::error::DomainError::ValidationError("Job not found".to_string())
            })?;

        job.complete_success(result_url);
        self.job_repository.update(&job).await?;

        Ok(())
    }

    async fn complete_job_failure(
        &self,
        job_id: &str,
        errors: Vec<JobError>,
    ) -> Result<(), crate::shared::error::DomainError> {
        let mut job = self
            .job_repository
            .find_by_job_id(job_id)
            .await?
            .ok_or_else(|| {
                crate::shared::error::DomainError::ValidationError("Job not found".to_string())
            })?;

        job.complete_failure(errors);
        self.job_repository.update(&job).await?;

        Ok(())
    }

    async fn complete_job_partial_success(
        &self,
        job_id: &str,
        result_url: Option<String>,
        errors: Vec<JobError>,
    ) -> Result<(), crate::shared::error::DomainError> {
        let mut job = self
            .job_repository
            .find_by_job_id(job_id)
            .await?
            .ok_or_else(|| {
                crate::shared::error::DomainError::ValidationError("Job not found".to_string())
            })?;

        job.complete_partial_success(result_url, errors);
        self.job_repository.update(&job).await?;

        Ok(())
    }

    async fn start_job_processing(
        &self,
        job_id: &str,
    ) -> Result<(), crate::shared::error::DomainError> {
        let mut job = self
            .job_repository
            .find_by_job_id(job_id)
            .await?
            .ok_or_else(|| {
                crate::shared::error::DomainError::ValidationError("Job not found".to_string())
            })?;

        job.start();
        self.job_repository.update(&job).await?;

        Ok(())
    }

    async fn find_by_status(
        &self,
        tenant_id: Uuid,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Job>, crate::shared::error::DomainError> {
        self.job_repository
            .find_by_status(tenant_id, status, limit)
            .await
    }
}
