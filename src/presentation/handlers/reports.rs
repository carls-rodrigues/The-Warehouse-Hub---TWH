use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    get_low_stock_report::GetLowStockReportRequest,
    get_stock_valuation_report::GetStockValuationReportRequest,
};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LowStockQuery {
    pub threshold: Option<i32>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StockValuationQuery {
    pub location_id: Option<Uuid>,
    pub valuation_method: Option<String>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CursorMeta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct LowStockReportResponse {
    pub data: Vec<LowStockItem>,
    pub cursor: Option<CursorMeta>,
}

#[derive(Debug, Serialize)]
pub struct LowStockItem {
    pub item: serde_json::Value,
    pub stock: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct StockValuationReportResponse {
    pub data: Vec<StockValuationItem>,
    pub cursor: Option<CursorMeta>,
}

#[derive(Debug, Serialize)]
pub struct StockValuationItem {
    pub item: serde_json::Value,
    pub valuation: f64,
}

/// Get low stock report
pub async fn get_low_stock_report(
    State(state): State<AppState>,
    Query(query): Query<LowStockQuery>,
) -> Result<Json<LowStockReportResponse>, (StatusCode, Json<ErrorResponse>)> {
    let threshold = query.threshold.unwrap_or(10); // Default threshold of 10

    match state
        .get_low_stock_report_use_case
        .execute(GetLowStockReportRequest {
            threshold,
            limit: query.limit.unwrap_or(50),
            cursor: query.cursor,
        })
        .await
    {
        Ok(response) => {
            let cursor_meta = response.next_cursor.map(|cursor| CursorMeta {
                next_cursor: Some(cursor),
                has_more: true,
            });

            let data = response
                .items
                .into_iter()
                .map(|item| LowStockItem {
                    item: serde_json::to_value(&item.item).unwrap_or_default(),
                    stock: serde_json::to_value(&item.stock).unwrap_or_default(),
                })
                .collect();

            Ok(Json(LowStockReportResponse {
                data,
                cursor: cursor_meta,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "ReportGenerationError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

/// Get stock valuation report
pub async fn get_stock_valuation_report(
    State(state): State<AppState>,
    Query(query): Query<StockValuationQuery>,
) -> Result<Json<StockValuationReportResponse>, (StatusCode, Json<ErrorResponse>)> {
    let valuation_method = query.valuation_method.unwrap_or_else(|| "FIFO".to_string());

    // Validate valuation method
    if !["FIFO", "LIFO", "AVG"].contains(&valuation_method.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "InvalidValuationMethod".to_string(),
                message: "Valuation method must be one of: FIFO, LIFO, AVG".to_string(),
            }),
        ));
    }

    match state
        .get_stock_valuation_report_use_case
        .execute(GetStockValuationReportRequest {
            location_id: query.location_id,
            valuation_method,
            limit: query.limit.unwrap_or(50),
            cursor: query.cursor,
        })
        .await
    {
        Ok(response) => {
            let cursor_meta = response.next_cursor.map(|cursor| CursorMeta {
                next_cursor: Some(cursor),
                has_more: true,
            });

            let data = response
                .items
                .into_iter()
                .map(|item| StockValuationItem {
                    item: serde_json::to_value(&item.item).unwrap_or_default(),
                    valuation: item.valuation,
                })
                .collect();

            Ok(Json(StockValuationReportResponse {
                data,
                cursor: cursor_meta,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "ReportGenerationError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
