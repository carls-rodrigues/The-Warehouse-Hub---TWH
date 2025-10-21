use crate::domain::entities::export::{
    CreateExportResponse, CreateStockCsvExportRequest, ExportType, StockCsvExportPayload,
};
use crate::domain::entities::job::CreateJobRequest;
use crate::domain::services::job_service::JobService;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use std::sync::Arc;

/// Service for handling data exports
#[async_trait]
pub trait ExportService: Send + Sync {
    async fn create_stock_csv_export(
        &self,
        request: CreateStockCsvExportRequest,
    ) -> Result<CreateExportResponse, DomainError>;
}

/// Implementation of ExportService
pub struct ExportServiceImpl<T: JobService> {
    job_service: Arc<T>,
}

impl<T: JobService> ExportServiceImpl<T> {
    pub fn new(job_service: Arc<T>) -> Self {
        Self { job_service }
    }
}

#[async_trait]
impl<T: JobService> ExportService for ExportServiceImpl<T>
where
    T: JobService,
{
    async fn create_stock_csv_export(
        &self,
        request: CreateStockCsvExportRequest,
    ) -> Result<CreateExportResponse, DomainError> {
        // Create job payload
        let payload = StockCsvExportPayload {
            location_id: request.location_id,
        };

        // Create job request
        let job_request = CreateJobRequest {
            job_type: "stock_csv_export".to_string(),
            payload: serde_json::to_value(payload).map_err(|e| {
                DomainError::ValidationError(format!("Failed to serialize payload: {}", e))
            })?,
        };

        // Enqueue job using the Jobs API
        let job = self
            .job_service
            .enqueue_job(request.tenant_id, job_request)
            .await?;

        Ok(CreateExportResponse {
            job_id: job.job_id.clone(),
            export_type: ExportType::StockCsv,
            status: job.status.to_string(),
            created_at: job.created_at,
        })
    }
}
