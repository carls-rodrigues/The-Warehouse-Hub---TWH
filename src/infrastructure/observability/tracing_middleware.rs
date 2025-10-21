use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use opentelemetry::trace::{SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{global, KeyValue};
use opentelemetry_semantic_conventions::attribute::{
    HTTP_METHOD, HTTP_STATUS_CODE, HTTP_URL, NETWORK_TRANSPORT,
};
use std::time::Instant;

use crate::infrastructure::observability::metrics::AppMetrics;

/// Middleware that creates OpenTelemetry spans for HTTP requests
pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let start_time = Instant::now();

    // Extract request details
    let method = request.method().clone();
    let uri = request.uri().clone();
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str())
        .unwrap_or(uri.path());

    // Create tracing span with OpenTelemetry attributes
    let span = tracing::span!(
        tracing::Level::INFO,
        "http_request",
        "http.method" = %method,
        "http.url" = %uri,
        "http.route" = matched_path,
        "network.transport" = "ip_tcp"
    );

    let _enter = span.enter();

    // Extract tenant context if available
    if let Some(tenant_context) =
        request
            .extensions()
            .get::<crate::infrastructure::middleware::tenant_middleware::TenantContext>()
    {
        tracing::Span::current().record(
            "tenant.id",
            &tracing::field::display(&tenant_context.tenant_id),
        );
        tracing::Span::current().record(
            "tenant.tier",
            &tracing::field::display(format!("{:?}", tenant_context.tier)),
        );
    }

    // Process request
    let response = next.run(request).await;

    // Record response details
    let status = response.status();
    tracing::Span::current().record(
        "http.status_code",
        &tracing::field::display(status.as_u16()),
    );

    // Record duration
    let duration = start_time.elapsed();
    tracing::Span::current().record(
        "http.duration_ms",
        &tracing::field::display(duration.as_millis()),
    );

    // Add error details if response indicates an error
    if !status.is_success() {
        tracing::Span::current().record("error", &tracing::field::display(true));
        tracing::Span::current().record(
            "error.message",
            &tracing::field::display(status.canonical_reason().unwrap_or("Unknown error")),
        );
    }

    // Record HTTP request metrics
    AppMetrics::get().record_http_request(
        &method.to_string(),
        status.as_u16(),
        duration.as_secs_f64(),
    );

    response
}
