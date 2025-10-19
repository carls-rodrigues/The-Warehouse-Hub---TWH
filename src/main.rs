use axum::{routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use std::env;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    db: String,
}

#[tokio::main]
async fn main() {
    // Initialize database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/twh".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Build the application with routes
    let app = Router::new()
        .route("/healthz", get(health_handler))
        .with_state(pool);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🚀 Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler(
    axum::extract::State(pool): axum::extract::State<PgPool>,
) -> Json<HealthResponse> {
    // Check database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "ok".to_string(),
        Err(_) => "down".to_string(),
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        db: db_status,
    })
}
