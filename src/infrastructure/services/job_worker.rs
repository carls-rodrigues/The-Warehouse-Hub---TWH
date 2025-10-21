use crate::domain::entities::job::{Job, JobError};
use crate::domain::services::{job_processor::JobProcessor, job_service::JobService};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use uuid;

pub struct WorkerManager<S: JobService, P: JobProcessor> {
    job_service: Arc<S>,
    job_processor: Arc<P>,
    job_queue: mpsc::Receiver<String>, // Receives job IDs
}

impl<S: JobService, P: JobProcessor> WorkerManager<S, P> {
    pub fn new(
        job_service: Arc<S>,
        job_processor: Arc<P>,
        job_queue: mpsc::Receiver<String>,
    ) -> Self {
        Self {
            job_service,
            job_processor,
            job_queue,
        }
    }

    pub async fn run(mut self) {
        info!("Starting job worker manager");

        while let Some(job_id) = self.job_queue.recv().await {
            info!("Processing job: {}", job_id);

            // For now, we'll process jobs one by one
            // In a real implementation, you'd have multiple workers
            if let Err(e) = self.process_single_job(&job_id).await {
                error!("Failed to process job {}: {:?}", job_id, e);
            }
        }

        info!("Job worker manager stopped");
    }

    async fn process_single_job(
        &self,
        job_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For simplicity, we'll process jobs for all tenants
        // In a real implementation, you'd have tenant-specific workers
        let tenant_id = uuid::Uuid::new_v4(); // This should be passed or retrieved properly
        let queued_jobs = self
            .job_service
            .find_by_status(tenant_id, "QUEUED", 10)
            .await?;

        for job in queued_jobs {
            if job.job_id == job_id {
                // Start processing the job
                self.job_service.start_job_processing(&job.job_id).await?;

                // Process the job
                match self.job_processor.process_job(&job).await {
                    Ok(_) => {
                        // Job completed successfully
                        self.job_service
                            .complete_job_success(&job.job_id, None)
                            .await?;
                        info!("Job {} completed successfully", job_id);
                    }
                    Err(e) => {
                        // Job failed
                        let errors = vec![e];
                        self.job_service
                            .complete_job_failure(&job.job_id, errors)
                            .await?;
                        error!("Job {} failed", job_id);
                    }
                }
                break;
            }
        }

        Ok(())
    }
}

// Basic job processor implementation
pub struct BasicJobProcessor;

impl BasicJobProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl JobProcessor for BasicJobProcessor {
    async fn process_job(&self, job: &Job) -> Result<(), JobError> {
        info!("Processing job: {} of type: {}", job.job_id, job.job_type);

        // Simulate job processing based on type
        match job.job_type.as_str() {
            "import_items" => {
                // Simulate importing items
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                info!("Successfully imported items for job {}", job.job_id);
            }
            "export_stock" => {
                // Simulate exporting stock
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                info!("Successfully exported stock for job {}", job.job_id);
            }
            "process_data" => {
                // Simulate processing data
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                info!("Successfully processed data for job {}", job.job_id);
            }
            _ => {
                return Err(JobError {
                    row: None,
                    message: format!("Unknown job type: {}", job.job_type),
                });
            }
        }

        Ok(())
    }
}
