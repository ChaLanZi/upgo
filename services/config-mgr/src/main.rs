//! Config Management Service — configuration management with RNacos.
//!
//! Provides REST API for config CRUD, watch/subscribe, and cache management,
//! backed by RNacos (Nacos-compatible configuration server).
//!
//! Usage:
//!   RNACOS_ADDR=http://rnacos:8848 cargo run -p config-mgr

mod config;
mod http;
mod rnacos_client;

use std::sync::Arc;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::AppConfig::from_env()?;

    let _otel = telemetry::init(telemetry::TelemetryConfig {
        service_name: "config-mgr".into(),
        otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://signoz-otel-collector:4317".into()),
        log_level: config.log_level.clone(),
    })?;

    tracing::info!("Starting Config Manager Service...");
    tracing::info!("HTTP endpoint: {}", config.http_addr);
    tracing::info!("RNacos endpoint: {}", config.rnacos_addr);

    let rnacos = Arc::new(rnacos_client::RnacosClient::new(&config));
    tracing::info!("Connected to RNacos");

    let app = http::routes(rnacos);

    let listener = tokio::net::TcpListener::bind(&config.http_addr).await?;
    tracing::info!("HTTP server listening on {}", config.http_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
