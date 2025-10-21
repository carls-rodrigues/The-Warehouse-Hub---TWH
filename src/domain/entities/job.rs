use crate::shared::error::DomainError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobError {
    pub row: Option<i32>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub job_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    Success,
    Failed,
    PartialSuccess,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "QUEUED"),
            JobStatus::Running => write!(f, "RUNNING"),
            JobStatus::Success => write!(f, "SUCCESS"),
            JobStatus::Failed => write!(f, "FAILED"),
            JobStatus::PartialSuccess => write!(f, "PARTIAL_SUCCESS"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_id: String,
    pub tenant_id: Uuid,
    pub job_type: String,
    pub status: JobStatus,
    pub progress: i32,
    pub payload: Option<serde_json::Value>,
    pub result_url: Option<String>,
    pub errors: Option<Vec<JobError>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Job {
    pub fn new(
        tenant_id: Uuid,
        job_type: String,
        payload: Option<serde_json::Value>,
    ) -> Result<Self, DomainError> {
        if job_type.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Job type cannot be empty".to_string(),
            ));
        }

        let job_id = format!("job_{}", Uuid::new_v4().simple());
        let now = chrono::Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            job_id,
            tenant_id,
            job_type,
            status: JobStatus::Queued,
            progress: 0,
            payload,
            result_url: None,
            errors: None,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
        })
    }

    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn complete_success(&mut self, result_url: Option<String>) {
        self.status = JobStatus::Success;
        self.progress = 100;
        self.result_url = result_url;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn complete_failure(&mut self, errors: Vec<JobError>) {
        self.status = JobStatus::Failed;
        self.errors = Some(errors);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn complete_partial_success(&mut self, result_url: Option<String>, errors: Vec<JobError>) {
        self.status = JobStatus::PartialSuccess;
        self.progress = 100;
        self.result_url = result_url;
        self.errors = Some(errors);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn update_progress(&mut self, progress: i32) {
        self.progress = progress.clamp(0, 100);
        self.updated_at = Utc::now();
    }
}
