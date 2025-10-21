use crate::domain::entities::job::{Job, JobError};
use async_trait::async_trait;

#[async_trait]
pub trait JobProcessor: Send + Sync {
    async fn process_job(&self, job: &Job) -> Result<(), JobError>;
}