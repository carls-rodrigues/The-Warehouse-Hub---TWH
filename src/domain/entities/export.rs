use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Export job types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    StockCsv,
}

/// Request to create a stock CSV export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStockCsvExportRequest {
    pub tenant_id: Uuid,
    pub location_id: Option<Uuid>, // Optional filter by location
}

/// Response from creating an export job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExportResponse {
    pub job_id: String,
    pub export_type: ExportType,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Export job payload for stock CSV
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockCsvExportPayload {
    pub location_id: Option<Uuid>,
}

/// CSV export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvExportResult {
    pub filename: String,
    pub record_count: i32,
    pub file_size_bytes: i64,
}
