pub mod metrics;
pub mod tracing_middleware;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::BatchSpanProcessor;
use opentelemetry_sdk::Resource;
use std::env;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(
            PeriodicReader::builder(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otlp_endpoint)
                    .build_metrics_exporter(
                        Box::new(
                            opentelemetry_sdk::metrics::reader::DefaultAggregationSelector::new(),
                        ),
                        Box::new(
                            opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector::new(),
                        ),
                    )?,
                opentelemetry_sdk::runtime::Tokio,
            )
            .build(),
        )
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

/// Shutdown OpenTelemetry gracefully
pub fn shutdown_observability() {
    tracing::info!("Shutting down OpenTelemetry");
    global::shutdown_tracer_provider();
}
