// Presentation layer routes
pub mod jobs;
pub mod purchase_order;
pub mod reports;
pub mod returns;
pub mod sales_order;
pub mod search;
pub mod stock;
pub mod tenant;
pub mod transfer;
pub mod webhook;

pub use jobs::create_jobs_routes;
pub use purchase_order::create_purchase_order_routes;
pub use reports::create_reports_routes;
pub use returns::return_routes;
pub use sales_order::sales_order_routes;
pub use stock::create_stock_routes;
pub use tenant::tenant_routes;
pub use transfer::transfer_routes;
pub use webhook::create_webhook_routes;
