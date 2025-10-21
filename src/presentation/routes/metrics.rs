use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};

use crate::infrastructure::observability::get_prometheus_registry;
use crate::AppState;

/// Create the metrics router
pub fn create_metrics_router() -> Router<AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}

/// Handler for the /metrics endpoint
async fn metrics_handler() -> String {
    let registry = get_prometheus_registry();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[cfg(test)]
mod tests {
    use super::metrics_handler;

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await;

        // The response should be a non-empty string
        assert!(!response.is_empty());

        // Should contain Prometheus format headers
        assert!(response.contains("# HELP"));
        assert!(response.contains("# TYPE"));

        // Should contain some basic metrics
        println!("Metrics response:\n{}", response);
    }
}
