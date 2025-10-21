use opentelemetry::metrics::{Counter, Histogram, ObservableGauge};
use std::sync::OnceLock;

/// Global metrics instance
static METRICS: OnceLock<AppMetrics> = OnceLock::new();

/// Application metrics
#[derive(Clone)]
pub struct AppMetrics {
    /// HTTP request counter
    pub http_requests_total: Counter<u64>,
    /// HTTP request duration histogram
    pub http_request_duration: Histogram<f64>,
    /// Database query counter
    pub db_queries_total: Counter<u64>,
    /// Database query duration histogram
    pub db_query_duration: Histogram<f64>,
    /// Active database connections gauge
    pub db_connections_active: ObservableGauge<u64>,
    /// Rate limit hits counter
    pub rate_limit_hits_total: Counter<u64>,
    /// Webhook delivery attempts counter
    pub webhook_deliveries_total: Counter<u64>,
    /// Job processing counter
    pub jobs_processed_total: Counter<u64>,
}

impl AppMetrics {
    /// Initialize metrics
    pub fn init() -> Self {
        let meter = opentelemetry::global::meter("warehouse-hub");

        let http_requests_total = meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests")
            .with_unit("requests")
            .init();

        let http_request_duration = meter
            .f64_histogram("http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .with_unit("s")
            .init();

        let db_queries_total = meter
            .u64_counter("db_queries_total")
            .with_description("Total number of database queries")
            .with_unit("queries")
            .init();

        let db_query_duration = meter
            .f64_histogram("db_query_duration_seconds")
            .with_description("Database query duration in seconds")
            .with_unit("s")
            .init();

        let db_connections_active = meter
            .u64_observable_gauge("db_connections_active")
            .with_description("Number of active database connections")
            .with_unit("connections")
            .init();

        let rate_limit_hits_total = meter
            .u64_counter("rate_limit_hits_total")
            .with_description("Total number of rate limit hits")
            .with_unit("hits")
            .init();

        let webhook_deliveries_total = meter
            .u64_counter("webhook_deliveries_total")
            .with_description("Total number of webhook delivery attempts")
            .with_unit("deliveries")
            .init();

        let jobs_processed_total = meter
            .u64_counter("jobs_processed_total")
            .with_description("Total number of jobs processed")
            .with_unit("jobs")
            .init();

        let metrics = Self {
            http_requests_total,
            http_request_duration,
            db_queries_total,
            db_query_duration,
            db_connections_active,
            rate_limit_hits_total,
            webhook_deliveries_total,
            jobs_processed_total,
        };

        METRICS.set(metrics.clone()).unwrap_or_else(|_| {
            panic!("Metrics already initialized");
        });

        metrics
    }

    /// Get the global metrics instance
    pub fn get() -> &'static Self {
        METRICS.get().expect("Metrics not initialized")
    }

    /// Record an HTTP request
    pub fn record_http_request(&self, method: &str, status: u16, duration: f64) {
        let attributes = vec![
            opentelemetry::KeyValue::new("method", method.to_string()),
            opentelemetry::KeyValue::new("status", status.to_string()),
        ];

        self.http_requests_total.add(1, &attributes);
        self.http_request_duration.record(duration, &attributes);
    }

    /// Record a database query
    pub fn record_db_query(&self, operation: &str, table: &str, duration: f64) {
        let attributes = vec![
            opentelemetry::KeyValue::new("operation", operation.to_string()),
            opentelemetry::KeyValue::new("table", table.to_string()),
        ];

        self.db_queries_total.add(1, &attributes);
        self.db_query_duration.record(duration, &attributes);
    }

    /// Record a rate limit hit
    pub fn record_rate_limit_hit(&self, tenant_id: &str, tier: &str) {
        let attributes = vec![
            opentelemetry::KeyValue::new("tenant_id", tenant_id.to_string()),
            opentelemetry::KeyValue::new("tier", tier.to_string()),
        ];

        self.rate_limit_hits_total.add(1, &attributes);
    }

    /// Record a webhook delivery attempt
    pub fn record_webhook_delivery(&self, status: &str, webhook_id: &str) {
        let attributes = vec![
            opentelemetry::KeyValue::new("status", status.to_string()),
            opentelemetry::KeyValue::new("webhook_id", webhook_id.to_string()),
        ];

        self.webhook_deliveries_total.add(1, &attributes);
    }

    /// Record a job processing
    pub fn record_job_processed(&self, job_type: &str, status: &str) {
        let attributes = vec![
            opentelemetry::KeyValue::new("job_type", job_type.to_string()),
            opentelemetry::KeyValue::new("status", status.to_string()),
        ];

        self.jobs_processed_total.add(1, &attributes);
    }
}
