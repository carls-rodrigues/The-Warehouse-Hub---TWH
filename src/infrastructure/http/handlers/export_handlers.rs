use crate::domain::entities::export::{CreateExportResponse, CreateStockCsvExportRequest};
use crate::domain::services::export_service::ExportService;
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};

/// Handler for creating a stock CSV export
pub async fn create_stock_csv_export(
    State(state): State<AppState>,
    Json(request): Json<CreateStockCsvExportRequest>,
) -> Result<Json<CreateExportResponse>, (StatusCode, String)> {
    match state.export_service.create_stock_csv_export(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create export: {}", e),
        )),
    }
}
