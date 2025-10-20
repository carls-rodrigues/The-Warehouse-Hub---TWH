// Presentation layer routes
pub mod purchase_order;
pub mod search;
pub mod stock;

pub use purchase_order::create_purchase_order_routes;
pub use stock::create_stock_routes;
