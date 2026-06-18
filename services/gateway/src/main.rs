//! Rust API Gateway (replaces Nginx).
//!
//! Routes:
//! - `/api/auth/*`   → reverse proxy to auth backend
//! - `/api/files/*`  → reverse proxy to files (FRS) backend
//! - `/*`            → serve embedded frontend static files
//!
//! Usage:
//!   AUTH_BACKEND=http://auth:50052 FILES_BACKEND=http://frs:9094 cargo run -p gateway

use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::routing::any;
use axum::Router;
use tower_http::cors::CorsLayer;

mod proxy;
mod static_files;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()))
        .init();

    let auth_backend = Arc::new(
        std::env::var("AUTH_BACKEND")
            .unwrap_or_else(|_| "http://auth.upgo.svc.cluster.local:50052".to_string()),
    );
    let files_backend = Arc::new(
        std::env::var("FILES_BACKEND")
            .unwrap_or_else(|_| "http://frs.upgo.svc.cluster.local:9094".to_string()),
    );
    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:80".to_string());

    tracing::info!("Gateway starting on {}", listen_addr);
    tracing::info!("Auth backend: {}", auth_backend);
    tracing::info!("Files backend: {}", files_backend);

    let app = Router::new()
        .fallback(any(move |req: Request<Body>| {
            let auth = auth_backend.clone();
            let files = files_backend.clone();
            async move { root_handler(req, auth, files).await }
        }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_handler(
    req: Request<Body>,
    auth_backend: Arc<String>,
    files_backend: Arc<String>,
) -> Response<Body> {
    let path = req.uri().path().to_string();

    if path.starts_with("/api/auth/") {
        proxy::api_proxy(req, &auth_backend).await
    } else if path.starts_with("/api/files/") {
        proxy::api_proxy(req, &files_backend).await
    } else if path.starts_with("/api/") {
        // fallback for any /api/* routes not matched above
        proxy::api_proxy(req, &auth_backend).await
    } else {
        static_files::serve(&path)
    }
}
