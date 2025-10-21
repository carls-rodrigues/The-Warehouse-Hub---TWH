mod application;
mod domain;
mod infrastructure;
mod presentation;
mod shared;

use crate::application::use_cases::{
    adjust_stock::AdjustStockUseCase, create_item::CreateItemUseCase,
    create_location::CreateLocationUseCase, create_purchase_order::CreatePurchaseOrderUseCase,
    create_return::CreateReturnUseCase, create_sales_order::CreateSalesOrderUseCase,
    create_transfer::CreateTransferUseCase, delete_item::DeleteItemUseCase,
    delete_location::DeleteLocationUseCase, enqueue_job::EnqueueJobUseCase,
    get_item::GetItemUseCase, get_job_status::GetJobStatusUseCase,
    get_location::GetLocationUseCase, get_low_stock_report::GetLowStockReportUseCase,
    get_purchase_order::GetPurchaseOrderUseCase, get_return::GetReturnUseCase,
    get_stock_level::GetStockLevelUseCase, get_stock_movements::GetStockMovementsUseCase,
    get_stock_valuation_report::GetStockValuationReportUseCase,
    list_item_stock_levels::ListItemStockLevelsUseCase, list_items::ListItemsUseCase,
    list_locations::ListLocationsUseCase, login::LoginUseCase,
    process_return::ProcessReturnUseCase, receive_purchase_order::ReceivePurchaseOrderUseCase,
    receive_transfer::ReceiveTransferUseCase, search_use_case::SearchUseCaseImpl,
    ship_sales_order::ShipSalesOrderUseCase, ship_transfer::ShipTransferUseCase,
    update_item::UpdateItemUseCase, update_location::UpdateLocationUseCase,
};
use crate::domain::services::export_service::{ExportService, ExportServiceImpl};
use crate::domain::services::webhook_dispatcher::{WebhookDispatcher, WebhookDispatcherImpl};
use crate::infrastructure::controllers::{
    auth_controller::login_handler, items_controller::*, locations_controller::*,
};
use crate::infrastructure::http::routes::export_routes;
use crate::infrastructure::repositories::{
    postgres_item_repository::PostgresItemRepository,
    postgres_job_repository::PostgresJobRepository,
    postgres_location_repository::PostgresLocationRepository,
    postgres_purchase_order_repository::PostgresPurchaseOrderRepository,
    postgres_return_repository::PostgresReturnRepository,
    postgres_sales_order_repository::PostgresSalesOrderRepository,
    postgres_search_repository::PostgresSearchRepository,
    postgres_stock_repository::PostgresStockRepository,
    postgres_transfer_repository::PostgresTransferRepository,
    postgres_user_repository::PostgresUserRepository,
    postgres_webhook_repository::PostgresWebhookRepository,
};
use crate::infrastructure::services::{
    job_service_impl::JobServiceImpl, report_service_impl::ReportServiceImpl,
};
use crate::presentation::routes::{
    create_jobs_routes, create_purchase_order_routes, create_reports_routes, create_stock_routes,
    create_webhook_routes, returns::return_routes, sales_order::sales_order_routes,
    search::create_search_routes, transfer::transfer_routes,
};
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
    pub user_repository: Arc<PostgresUserRepository>,
    pub item_repository: Arc<PostgresItemRepository>,
    pub location_repository: Arc<PostgresLocationRepository>,
    pub purchase_order_repository: Arc<PostgresPurchaseOrderRepository>,
    pub return_repository: Arc<PostgresReturnRepository>,
    pub sales_order_repository: Arc<PostgresSalesOrderRepository>,
    pub transfer_repository: Arc<PostgresTransferRepository>,
    pub stock_repository: Arc<PostgresStockRepository>,
    pub search_repository: Arc<PostgresSearchRepository>,
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
    pub create_purchase_order_use_case: Arc<
        CreatePurchaseOrderUseCase<
            PostgresPurchaseOrderRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub get_purchase_order_use_case: Arc<GetPurchaseOrderUseCase<PostgresPurchaseOrderRepository>>,
    pub receive_purchase_order_use_case: Arc<
        ReceivePurchaseOrderUseCase<
            PostgresPurchaseOrderRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub create_return_use_case: Arc<
        CreateReturnUseCase<
            PostgresReturnRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub get_return_use_case: Arc<GetReturnUseCase<PostgresReturnRepository>>,
    pub process_return_use_case: Arc<ProcessReturnUseCase<PostgresReturnRepository>>,
    pub create_sales_order_use_case: Arc<
        CreateSalesOrderUseCase<
            PostgresSalesOrderRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub ship_sales_order_use_case: Arc<
        ShipSalesOrderUseCase<
            PostgresSalesOrderRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub create_transfer_use_case: Arc<
        CreateTransferUseCase<
            PostgresTransferRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub receive_transfer_use_case: Arc<
        ReceiveTransferUseCase<
            PostgresTransferRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub ship_transfer_use_case: Arc<
        ShipTransferUseCase<
            PostgresTransferRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub search_use_case: Arc<SearchUseCaseImpl<PostgresSearchRepository>>,
    pub get_stock_level_use_case: Arc<
        GetStockLevelUseCase<
            PostgresStockRepository,
            PostgresItemRepository,
            PostgresLocationRepository,
        >,
    >,
    pub list_item_stock_levels_use_case: Arc<
        ListItemStockLevelsUseCase<
            PostgresStockRepository,
            PostgresItemRepository,
            PostgresLocationRepository,
        >,
    >,
    pub get_stock_movements_use_case: Arc<
        GetStockMovementsUseCase<
            PostgresStockRepository,
            PostgresItemRepository,
            PostgresLocationRepository,
        >,
    >,
    pub adjust_stock_use_case: Arc<
        AdjustStockUseCase<
            PostgresStockRepository,
            WebhookDispatcherImpl<PostgresWebhookRepository>,
        >,
    >,
    pub webhook_repository: Arc<PostgresWebhookRepository>,
    pub webhook_dispatcher: Arc<WebhookDispatcherImpl<PostgresWebhookRepository>>,
    pub report_service: Arc<ReportServiceImpl<PostgresItemRepository, PostgresStockRepository>>,
    pub get_low_stock_report_use_case: Arc<
        GetLowStockReportUseCase<
            PostgresItemRepository,
            PostgresStockRepository,
            ReportServiceImpl<PostgresItemRepository, PostgresStockRepository>,
        >,
    >,
    pub get_stock_valuation_report_use_case: Arc<
        GetStockValuationReportUseCase<
            ReportServiceImpl<PostgresItemRepository, PostgresStockRepository>,
        >,
    >,
    pub job_repository: Arc<PostgresJobRepository>,
    pub job_service: Arc<JobServiceImpl<PostgresJobRepository>>,
    pub enqueue_job_use_case: Arc<EnqueueJobUseCase<JobServiceImpl<PostgresJobRepository>>>,
    pub get_job_status_use_case: Arc<GetJobStatusUseCase<JobServiceImpl<PostgresJobRepository>>>,
    pub export_service: Arc<ExportServiceImpl<JobServiceImpl<PostgresJobRepository>>>,
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
    let purchase_order_repository =
        Arc::new(PostgresPurchaseOrderRepository::new(Arc::clone(&pool)));
    let return_repository = Arc::new(PostgresReturnRepository::new(Arc::clone(&pool)));
    let sales_order_repository = Arc::new(PostgresSalesOrderRepository::new(Arc::clone(&pool)));
    let transfer_repository = Arc::new(PostgresTransferRepository::new(Arc::clone(&pool)));
    let search_repository = Arc::new(PostgresSearchRepository::new(Arc::clone(&pool)));
    let stock_repository = Arc::new(PostgresStockRepository::new(Arc::clone(&pool)));

    let webhook_repository = Arc::new(PostgresWebhookRepository::new(Arc::clone(&pool)));
    let webhook_dispatcher = Arc::new(WebhookDispatcherImpl::new(Arc::clone(&webhook_repository)));

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

    let create_purchase_order_use_case = Arc::new(CreatePurchaseOrderUseCase::new(
        Arc::clone(&purchase_order_repository),
        Arc::clone(&webhook_dispatcher),
    ));
    let get_purchase_order_use_case = Arc::new(GetPurchaseOrderUseCase::new(Arc::clone(
        &purchase_order_repository,
    )));
    let receive_purchase_order_use_case = Arc::new(ReceivePurchaseOrderUseCase::new(
        Arc::clone(&purchase_order_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    let create_return_use_case = Arc::new(CreateReturnUseCase::new(
        Arc::clone(&return_repository),
        Arc::clone(&webhook_dispatcher),
    ));
    let get_return_use_case = Arc::new(GetReturnUseCase::new(Arc::clone(&return_repository)));
    let process_return_use_case =
        Arc::new(ProcessReturnUseCase::new(Arc::clone(&return_repository)));

    let create_sales_order_use_case = Arc::new(CreateSalesOrderUseCase::new(
        Arc::clone(&sales_order_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    let ship_sales_order_use_case = Arc::new(ShipSalesOrderUseCase::new(
        Arc::clone(&sales_order_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    let create_transfer_use_case = Arc::new(CreateTransferUseCase::new(
        Arc::clone(&transfer_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    let receive_transfer_use_case = Arc::new(ReceiveTransferUseCase::new(
        Arc::clone(&transfer_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    let ship_transfer_use_case = Arc::new(ShipTransferUseCase::new(
        Arc::clone(&transfer_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    let search_use_case = Arc::new(SearchUseCaseImpl::new(Arc::clone(&search_repository)));

    let get_stock_level_use_case = Arc::new(GetStockLevelUseCase::new(
        Arc::clone(&stock_repository),
        Arc::clone(&item_repository),
        Arc::clone(&location_repository),
    ));
    let list_item_stock_levels_use_case = Arc::new(ListItemStockLevelsUseCase::new(
        Arc::clone(&stock_repository),
        Arc::clone(&item_repository),
        Arc::clone(&location_repository),
    ));
    let get_stock_movements_use_case = Arc::new(GetStockMovementsUseCase::new(
        Arc::clone(&stock_repository),
        Arc::clone(&item_repository),
        Arc::clone(&location_repository),
    ));
    let adjust_stock_use_case = Arc::new(AdjustStockUseCase::new(
        Arc::clone(&stock_repository),
        Arc::clone(&webhook_dispatcher),
    ));

    // Initialize report service and use cases
    let report_service = Arc::new(ReportServiceImpl::new(
        Arc::clone(&item_repository),
        Arc::clone(&stock_repository),
    ));
    let get_low_stock_report_use_case = Arc::new(GetLowStockReportUseCase::new(
        Arc::clone(&item_repository),
        Arc::clone(&stock_repository),
        Arc::clone(&report_service),
    ));
    let get_stock_valuation_report_use_case = Arc::new(GetStockValuationReportUseCase::new(
        Arc::clone(&report_service),
    ));

    // Initialize job repository and service
    let job_repository = Arc::new(PostgresJobRepository::new(Arc::clone(&pool)));
    let job_service = Arc::new(JobServiceImpl::new(Arc::clone(&job_repository)));
    let enqueue_job_use_case = Arc::new(EnqueueJobUseCase::new(Arc::clone(&job_service)));
    let get_job_status_use_case = Arc::new(GetJobStatusUseCase::new(Arc::clone(&job_service)));

    // Initialize export service
    let export_service = Arc::new(ExportServiceImpl::new(Arc::clone(&job_service)));

    let app_state = AppState {
        pool: Arc::clone(&pool),
        user_repository: Arc::clone(&user_repository),
        item_repository: Arc::clone(&item_repository),
        location_repository: Arc::clone(&location_repository),
        purchase_order_repository: Arc::clone(&purchase_order_repository),
        return_repository: Arc::clone(&return_repository),
        sales_order_repository: Arc::clone(&sales_order_repository),
        transfer_repository: Arc::clone(&transfer_repository),
        stock_repository: Arc::clone(&stock_repository),
        search_repository: Arc::clone(&search_repository),
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
        create_purchase_order_use_case,
        get_purchase_order_use_case,
        receive_purchase_order_use_case,
        create_return_use_case,
        get_return_use_case,
        process_return_use_case,
        create_sales_order_use_case,
        ship_sales_order_use_case,
        create_transfer_use_case,
        receive_transfer_use_case,
        ship_transfer_use_case,
        search_use_case,
        get_stock_level_use_case,
        list_item_stock_levels_use_case,
        get_stock_movements_use_case,
        adjust_stock_use_case,
        webhook_repository,
        webhook_dispatcher,
        report_service,
        get_low_stock_report_use_case,
        get_stock_valuation_report_use_case,
        job_repository: Arc::clone(&job_repository),
        job_service: Arc::clone(&job_service),
        enqueue_job_use_case: Arc::clone(&enqueue_job_use_case),
        get_job_status_use_case: Arc::clone(&get_job_status_use_case),
        export_service: Arc::clone(&export_service),
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
        .merge(create_stock_routes())
        .merge(create_reports_routes())
        .merge(create_jobs_routes())
        .merge(create_purchase_order_routes())
        .merge(sales_order_routes())
        .merge(transfer_routes())
        .merge(return_routes())
        .merge(create_webhook_routes())
        .merge(export_routes::create_exports_router())
        .with_state(app_state);

    // Run the server
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("🚀 Server running on http://{addr}");
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
