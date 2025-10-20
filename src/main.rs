mod application;
mod domain;
mod infrastructure;
mod presentation;
mod shared;

use crate::application::use_cases::{
    create_item::CreateItemUseCase, create_location::CreateLocationUseCase,
    delete_item::DeleteItemUseCase, delete_location::DeleteLocationUseCase,
    get_item::GetItemUseCase, get_location::GetLocationUseCase, list_items::ListItemsUseCase,
    list_locations::ListLocationsUseCase, login::LoginUseCase, search_use_case::SearchUseCaseImpl,
    update_item::UpdateItemUseCase, update_location::UpdateLocationUseCase,
};
use crate::infrastructure::controllers::{
    auth_controller::login_handler, items_controller::*, locations_controller::*,
};
use crate::infrastructure::repositories::{
    postgres_item_repository::PostgresItemRepository,
    postgres_location_repository::PostgresLocationRepository,
    postgres_search_repository::PostgresSearchRepository,
    postgres_user_repository::PostgresUserRepository,
};
use crate::presentation::routes::search::create_search_routes;
use axum::{
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    pub login_use_case: Arc<LoginUseCase<PostgresUserRepository>>,
    pub create_item_use_case: Arc<CreateItemUseCase<PostgresItemRepository>>,
    pub get_item_use_case: Arc<GetItemUseCase<PostgresItemRepository>>,
    pub update_item_use_case: Arc<UpdateItemUseCase<PostgresItemRepository>>,
    pub list_items_use_case: Arc<ListItemsUseCase<PostgresItemRepository>>,
    pub delete_item_use_case: Arc<DeleteItemUseCase<PostgresItemRepository>>,
    pub create_location_use_case: Arc<CreateLocationUseCase<PostgresLocationRepository>>,
    pub get_location_use_case: Arc<GetLocationUseCase<PostgresLocationRepository>>,
    pub update_location_use_case: Arc<UpdateLocationUseCase<PostgresLocationRepository>>,
    pub list_locations_use_case: Arc<ListLocationsUseCase<PostgresLocationRepository>>,
    pub delete_location_use_case: Arc<DeleteLocationUseCase<PostgresLocationRepository>>,
    pub search_use_case: Arc<SearchUseCaseImpl<PostgresSearchRepository>>,
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
    let item_repository = Arc::new(PostgresItemRepository::new(Arc::clone(&pool)));
    let location_repository = Arc::new(PostgresLocationRepository::new(Arc::clone(&pool)));
    let search_repository = Arc::new(PostgresSearchRepository::new(Arc::clone(&pool)));

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

    let create_item_use_case = Arc::new(CreateItemUseCase::new(Arc::clone(&item_repository)));
    let get_item_use_case = Arc::new(GetItemUseCase::new(Arc::clone(&item_repository)));
    let update_item_use_case = Arc::new(UpdateItemUseCase::new(Arc::clone(&item_repository)));
    let list_items_use_case = Arc::new(ListItemsUseCase::new(Arc::clone(&item_repository)));
    let delete_item_use_case = Arc::new(DeleteItemUseCase::new(Arc::clone(&item_repository)));

    let create_location_use_case =
        Arc::new(CreateLocationUseCase::new(Arc::clone(&location_repository)));
    let get_location_use_case = Arc::new(GetLocationUseCase::new(Arc::clone(&location_repository)));
    let update_location_use_case =
        Arc::new(UpdateLocationUseCase::new(Arc::clone(&location_repository)));
    let list_locations_use_case =
        Arc::new(ListLocationsUseCase::new(Arc::clone(&location_repository)));
    let delete_location_use_case =
        Arc::new(DeleteLocationUseCase::new(Arc::clone(&location_repository)));

    let search_use_case = Arc::new(SearchUseCaseImpl::new(Arc::clone(&search_repository)));

    let app_state = AppState {
        pool: Arc::clone(&pool),
        login_use_case,
        create_item_use_case,
        get_item_use_case,
        update_item_use_case,
        list_items_use_case,
        delete_item_use_case,
        create_location_use_case,
        get_location_use_case,
        update_location_use_case,
        list_locations_use_case,
        delete_location_use_case,
        search_use_case,
    };

    // Build the application with routes
    let app = Router::new()
        .route("/healthz", get(health_handler))
        .route("/auth/login", post(login_handler))
        .route("/items", post(create_item_handler))
        .route("/items", get(list_items_handler))
        .route("/items/{id}", get(get_item_handler))
        .route("/items/{id}", put(update_item_handler))
        .route("/items/{id}", delete(delete_item_handler))
        .route("/locations", post(create_location_handler))
        .route("/locations", get(list_locations_handler))
        .route("/locations/{id}", get(get_location_handler))
        .route("/locations/{id}", put(update_location_handler))
        .route("/locations/{id}", delete(delete_location_handler))
        .merge(create_search_routes())
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
