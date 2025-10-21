use crate::domain::entities::job::Job;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Find a job by its job_id
    async fn find_by_job_id(&self, job_id: &str) -> Result<Option<Job>, DomainError>;

    /// Find a job by its internal ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, DomainError>;

    /// Save a new job
    async fn save(&self, job: &Job) -> Result<(), DomainError>;

    /// Update an existing job
    async fn update(&self, job: &Job) -> Result<(), DomainError>;

    /// List jobs for a tenant with pagination
    async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Job>, DomainError>;

    /// Count jobs for a tenant
    async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<i64, DomainError>;

    /// Find jobs by status for a tenant
    async fn find_by_status(
        &self,
        tenant_id: Uuid,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Job>, DomainError>;
}
