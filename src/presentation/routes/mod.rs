// Presentation layer routes
pub mod admin;
pub mod jobs;
pub mod metrics;
pub mod purchase_order;
pub mod reports;
pub mod returns;
pub mod sales_order;
pub mod search;
pub mod stock;
pub mod tenant;
pub mod transfer;
pub mod webhook;

pub use admin::create_admin_router;
pub use jobs::create_jobs_routes;
pub use metrics::create_metrics_router;
pub use purchase_order::create_purchase_order_routes;
pub use reports::create_reports_routes;
pub use returns::return_routes;
pub use sales_order::sales_order_routes;
pub use stock::create_stock_routes;
pub use tenant::tenant_routes;
pub use transfer::transfer_routes;
pub use webhook::create_webhook_routes;
