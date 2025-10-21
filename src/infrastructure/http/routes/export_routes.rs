use crate::infrastructure::http::handlers::export_handlers;
use crate::AppState;
use axum::{routing::post, Router};

pub fn create_exports_router() -> Router<AppState> {
    Router::new().route(
        "/exports/stock_csv",
        post(export_handlers::create_stock_csv_export),
    )
}
