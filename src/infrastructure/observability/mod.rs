pub mod metrics;
pub mod tracing_middleware;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::Resource;
use prometheus::Registry;
use std::env;
use std::sync::Arc;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Global Prometheus registry for metrics exposition
static PROMETHEUS_REGISTRY: std::sync::OnceLock<Arc<Registry>> = std::sync::OnceLock::new();

/// Initialize OpenTelemetry tracing and metrics
pub fn init_observability() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get OTLP endpoint from environment, default to localhost
    let otlp_endpoint =
        env::var("OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

    // Create resource with service information
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", "warehouse-hub"),
        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    // Initialize tracing
    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otlp_endpoint.clone())
                    .build_span_exporter()?,
                opentelemetry_sdk::runtime::Tokio,
            )
            .build(),
        )
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    // Initialize metrics
    let registry = Registry::new();
    let prometheus_exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()?;

    PROMETHEUS_REGISTRY.set(Arc::new(registry)).unwrap();

    let meter_provider = SdkMeterProvider::builder()
        .with_reader(prometheus_exporter)
        .build();

    global::set_meter_provider(meter_provider);

    // Initialize tracing subscriber with OpenTelemetry layer
    let tracer = tracer_provider.tracer("warehouse-hub");
    let telemetry = OpenTelemetryLayer::new(tracer);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("warehouse_hub=info".parse()?)
                .add_directive("axum=info".parse()?)
                .add_directive("sqlx=warn".parse()?),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .with(telemetry);

    subscriber.init();

    tracing::info!("OpenTelemetry initialized successfully");
    Ok(())
}

/// Get the global Prometheus registry
pub fn get_prometheus_registry() -> Arc<Registry> {
    Arc::clone(
        PROMETHEUS_REGISTRY
            .get()
            .expect("Prometheus registry not initialized"),
    )
}
