use crate::presentation::handlers::reports::{get_low_stock_report, get_stock_valuation_report};
use crate::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn create_reports_routes() -> Router<AppState> {
    Router::new()
        .route("/reports/low_stock", get(get_low_stock_report))
        .route("/reports/stock_valuation", get(get_stock_valuation_report))
}
