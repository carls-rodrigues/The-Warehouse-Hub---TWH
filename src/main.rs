mod application;
mod domain;
mod infrastructure;
mod shared;

use crate::application::use_cases::login::LoginUseCase;
use crate::infrastructure::controllers::auth_controller::login_handler;
use crate::infrastructure::repositories::postgres_user_repository::PostgresUserRepository;
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    pub login_use_case: Arc<LoginUseCase<PostgresUserRepository>>,
}
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    db: String,
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize database connection
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/twh".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let pool = Arc::new(pool);

    // Initialize dependencies
    let user_repository = Arc::new(PostgresUserRepository::new(Arc::clone(&pool)));

    // Get JWT configuration from environment
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let jwt_expiry_hours: i64 = env::var("JWT_EXPIRY_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .expect("JWT_EXPIRY_HOURS must be a valid number");

    let login_use_case = Arc::new(LoginUseCase::new(
        Arc::clone(&user_repository),
        jwt_secret,
        jwt_expiry_hours,
    ));

    let app_state = AppState {
        pool: Arc::clone(&pool),
        login_use_case,
    };

    // Build the application with routes
    let app = Router::new()
        .route("/healthz", get(health_handler))
        .route("/auth/login", post(login_handler))
        .with_state(app_state);

    // Run the server
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<HealthResponse> {
    // Check database connectivity
    let db_status = match sqlx::query("SELECT 1").fetch_one(&*state.pool).await {
        Ok(_) => "ok".to_string(),
        Err(_) => "down".to_string(),
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        db: db_status,
    })
}
