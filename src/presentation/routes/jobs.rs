use crate::presentation::handlers::jobs::{enqueue_job, get_job_status};
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn create_jobs_routes() -> Router<AppState> {
    Router::new()
        .route("/jobs", post(enqueue_job))
        .route("/jobs/{jobId}", get(get_job_status))
}
