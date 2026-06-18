//! FRS (File Repository Service) — MinIO-based file management.
//!
//! Provides REST API for file upload/download/delete/list operations,
//! backed by S3-compatible storage (MinIO).
//!
//! Usage:
//!   cargo run -p frs

mod config;
mod http;
mod storage;

use std::sync::Arc;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::AppConfig::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    tracing::info!("Starting FRS (File Repository Service)...");
    tracing::info!("HTTP endpoint: {}", config.http_addr);
    tracing::info!("gRPC endpoint: {}", config.grpc_addr);
    tracing::info!("S3 endpoint: {}", config.s3_endpoint);
    tracing::info!("S3 bucket: {}", config.s3_bucket);

    // Initialize S3 storage
    let storage = storage::StorageService::new(&config).await?;
    tracing::info!("Connected to MinIO, bucket ready");

    // Build HTTP router
    let app = http::routes(Arc::new(storage));

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(&config.http_addr).await?;
    tracing::info!("HTTP server listening on {}", config.http_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
