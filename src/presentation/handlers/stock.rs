use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::use_cases::{
    adjust_stock::AdjustStockResponse, get_stock_level::GetStockLevelRequest,
    list_item_stock_levels::ListItemStockLevelsRequest,
};
use crate::domain::entities::inventory::StockAdjustmentRequest;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct StockMovementsQuery {
    pub item_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get stock level for a specific item at a specific location
pub async fn get_stock_level(
    State(state): State<AppState>,
    Path((item_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Option<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .get_stock_level_use_case
        .execute(GetStockLevelRequest {
            item_id,
            location_id,
        })
        .await
    {
        Ok(Some(stock_level)) => Ok(Json(Some(serde_json::to_value(stock_level).map_err(
            |e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "SerializationError".to_string(),
                        message: e.to_string(),
                    }),
                )
            },
        )?))),
        Ok(None) => Ok(Json(None)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "StockError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

/// Get all stock levels for a specific item across all locations
pub async fn get_item_stock_levels(
    State(state): State<AppState>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .list_item_stock_levels_use_case
        .execute(ListItemStockLevelsRequest { item_id })
        .await
    {
        Ok(response) => Ok(Json(serde_json::to_value(response).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "SerializationError".to_string(),
                    message: e.to_string(),
                }),
            )
        })?)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "StockError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

/// Get stock movements with optional filtering
pub async fn get_stock_movements(
    State(state): State<AppState>,
    Query(query): Query<StockMovementsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0).max(0);

    match state
        .get_stock_movements_use_case
        .execute(query.item_id, query.location_id, limit, offset)
        .await
    {
        Ok(movements) => {
            let json_movements = movements
                .into_iter()
                .map(|m| serde_json::to_value(m))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "SerializationError".to_string(),
                            message: e.to_string(),
                        }),
                    )
                })?;
            Ok(Json(json_movements))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "StockError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}

/// Adjust stock level (requires authentication)
pub async fn adjust_stock(
    State(state): State<AppState>,
    Json(request): Json<StockAdjustmentRequest>,
) -> Result<Json<AdjustStockResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Get user ID from authentication context
    let created_by = None; // For now, no authentication

    match state
        .adjust_stock_use_case
        .execute(request, created_by)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "StockError".to_string(),
                message: e.to_string(),
            }),
        )),
    }
}
