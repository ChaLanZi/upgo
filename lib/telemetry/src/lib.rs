//! upgo Telemetry — OpenTelemetry initialization shared library.
//!
//! Provides a single `init()` function that sets up:
//! - **Traces**: OTLP gRPC exporter → SigNoz OTEL Collector
//! - **Logs**: via tracing-subscriber (JSON format), with span context
//!
//! Usage:
//!
//! ```no_run
//! let _guard = telemetry::init(telemetry::TelemetryConfig {
//!     service_name: "my-service".into(),
//!     otlp_endpoint: "http://signoz-otel-collector:4317".into(),
//!     log_level: "info".into(),
//! }).unwrap();
//! ```

use anyhow::Result;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::{self as sdktrace, RandomIdGenerator};
use opentelemetry_sdk::Resource;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name (e.g. "auth", "gateway", "frs")
    pub service_name: String,
    /// OTLP endpoint (e.g. "http://signoz-otel-collector:4317")
    pub otlp_endpoint: String,
    /// Log level filter (e.g. "info", "debug")
    pub log_level: String,
}

/// Initialize OpenTelemetry tracing and logging.
/// Returns a guard that must be kept alive for the duration of the program.
pub fn init(config: TelemetryConfig) -> Result<TelemetryGuard> {
    let resource = Resource::new([
        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        opentelemetry::KeyValue::new("deployment.environment", "development"),
    ]);

    // ── Trace Provider ────────────────────────────────────
    let tracer_provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .build()?,
            opentelemetry_sdk::runtime::Tokio,
        )
        .with_resource(resource)
        .with_id_generator(RandomIdGenerator::default())
        .build();

    let tracer = tracer_provider.tracer(config.service_name.clone());

    // ── Tracing Subscriber ────────────────────────────────
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_current_span(true);

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    tracing::info!(
        service = %config.service_name,
        otlp = %config.otlp_endpoint,
        "Telemetry initialized",
    );

    Ok(TelemetryGuard { tracer_provider })
}

/// Guard that flushes telemetry on drop.
pub struct TelemetryGuard {
    tracer_provider: sdktrace::TracerProvider,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("TracerProvider shutdown error: {}", e);
        }
        opentelemetry::global::shutdown_tracer_provider();
    }
}
