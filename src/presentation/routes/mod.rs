// Presentation layer routes
pub mod purchase_order;
pub mod returns;
pub mod sales_order;
pub mod search;
pub mod stock;
pub mod transfer;

pub use purchase_order::create_purchase_order_routes;
pub use returns::return_routes;
pub use sales_order::sales_order_routes;
pub use stock::create_stock_routes;
pub use transfer::transfer_routes;
