use crate::domain::entities::job::{Job, JobError, JobStatus};
use crate::domain::services::job_repository::JobRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresJobRepository {
    pool: Arc<PgPool>,
}

impl PostgresJobRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for PostgresJobRepository {
    async fn find_by_job_id(&self, job_id: &str) -> Result<Option<Job>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, job_id, tenant_id, type, status, progress, payload, result_url, errors,
                   created_at, updated_at, started_at, completed_at
            FROM jobs
            WHERE job_id = $1
            "#,
            job_id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let status = match row.status.as_str() {
                    "QUEUED" => JobStatus::Queued,
                    "RUNNING" => JobStatus::Running,
                    "SUCCESS" => JobStatus::Success,
                    "FAILED" => JobStatus::Failed,
                    "PARTIAL_SUCCESS" => JobStatus::PartialSuccess,
                    _ => JobStatus::Queued, // Default fallback
                };

                let errors = row
                    .errors
                    .map(|e| serde_json::from_value(e).unwrap_or_default());

                Ok(Some(Job {
                    id: row.id,
                    job_id: row.job_id,
                    tenant_id: row.tenant_id,
                    job_type: row.r#type,
                    status,
                    progress: row.progress,
                    payload: row.payload,
                    result_url: row.result_url,
                    errors,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    started_at: row.started_at,
                    completed_at: row.completed_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, job_id, tenant_id, type, status, progress, payload, result_url, errors,
                   created_at, updated_at, started_at, completed_at
            FROM jobs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let status = match row.status.as_str() {
                    "QUEUED" => JobStatus::Queued,
                    "RUNNING" => JobStatus::Running,
                    "SUCCESS" => JobStatus::Success,
                    "FAILED" => JobStatus::Failed,
                    "PARTIAL_SUCCESS" => JobStatus::PartialSuccess,
                    _ => JobStatus::Queued,
                };

                let errors = row
                    .errors
                    .map(|e| serde_json::from_value(e).unwrap_or_default());

                Ok(Some(Job {
                    id: row.id,
                    job_id: row.job_id,
                    tenant_id: row.tenant_id,
                    job_type: row.r#type,
                    status,
                    progress: row.progress,
                    payload: row.payload,
                    result_url: row.result_url,
                    errors,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    started_at: row.started_at,
                    completed_at: row.completed_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, job: &Job) -> Result<(), DomainError> {
        let status_str = job.status.to_string();
        let errors_json = job
            .errors
            .as_ref()
            .map(|e| serde_json::to_value(e).unwrap_or_default());

        sqlx::query!(
            r#"
            INSERT INTO jobs (id, job_id, tenant_id, type, status, progress, payload, result_url, errors,
                             created_at, updated_at, started_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            job.id,
            job.job_id,
            job.tenant_id,
            job.job_type,
            status_str,
            job.progress,
            job.payload,
            job.result_url,
            errors_json,
            job.created_at,
            job.updated_at,
            job.started_at,
            job.completed_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update(&self, job: &Job) -> Result<(), DomainError> {
        let status_str = job.status.to_string();
        let errors_json = job
            .errors
            .as_ref()
            .map(|e| serde_json::to_value(e).unwrap_or_default());

        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $1, progress = $2, result_url = $3, errors = $4,
                updated_at = $5, started_at = $6, completed_at = $7
            WHERE id = $8
            "#,
            status_str,
            job.progress,
            job.result_url,
            errors_json,
            job.updated_at,
            job.started_at,
            job.completed_at,
            job.id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn list_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Job>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, job_id, tenant_id, type, status, progress, payload, result_url, errors,
                   created_at, updated_at, started_at, completed_at
            FROM jobs
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut jobs = Vec::new();
        for row in rows {
            let status = match row.status.as_str() {
                "QUEUED" => JobStatus::Queued,
                "RUNNING" => JobStatus::Running,
                "SUCCESS" => JobStatus::Success,
                "FAILED" => JobStatus::Failed,
                "PARTIAL_SUCCESS" => JobStatus::PartialSuccess,
                _ => JobStatus::Queued,
            };

            let errors = row
                .errors
                .map(|e| serde_json::from_value(e).unwrap_or_default());

            jobs.push(Job {
                id: row.id,
                job_id: row.job_id,
                tenant_id: row.tenant_id,
                job_type: row.r#type,
                status,
                progress: row.progress,
                payload: row.payload,
                result_url: row.result_url,
                errors,
                created_at: row.created_at,
                updated_at: row.updated_at,
                started_at: row.started_at,
                completed_at: row.completed_at,
            });
        }

        Ok(jobs)
    }

    async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<i64, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM jobs
            WHERE tenant_id = $1
            "#,
            tenant_id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(result.count.unwrap_or(0))
    }

    async fn find_by_status(
        &self,
        tenant_id: Uuid,
        status: &str,
        limit: i64,
    ) -> Result<Vec<Job>, DomainError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, job_id, tenant_id, type, status, progress, payload, result_url, errors,
                   created_at, updated_at, started_at, completed_at
            FROM jobs
            WHERE tenant_id = $1 AND status = $2
            ORDER BY created_at ASC
            LIMIT $3
            "#,
            tenant_id,
            status,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        let mut jobs = Vec::new();
        for row in rows {
            let job_status = match row.status.as_str() {
                "QUEUED" => JobStatus::Queued,
                "RUNNING" => JobStatus::Running,
                "SUCCESS" => JobStatus::Success,
                "FAILED" => JobStatus::Failed,
                "PARTIAL_SUCCESS" => JobStatus::PartialSuccess,
                _ => JobStatus::Queued,
            };

            let errors = row
                .errors
                .map(|e| serde_json::from_value(e).unwrap_or_default());

            jobs.push(Job {
                id: row.id,
                job_id: row.job_id,
                tenant_id: row.tenant_id,
                job_type: row.r#type,
                status: job_status,
                progress: row.progress,
                payload: row.payload,
                result_url: row.result_url,
                errors,
                created_at: row.created_at,
                updated_at: row.updated_at,
                started_at: row.started_at,
                completed_at: row.completed_at,
            });
        }

        Ok(jobs)
    }
}
