// Debug build 时隐藏开发期常见警告，release build 保持严格检查
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut,)
)]

use account::infrastructure::config::AppConfig;
use account::infrastructure::db::init_pool;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::from_env()?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    tracing::info!("Starting Account Service...");

    // Initialize database pool
    let db_pool = init_pool(&config.database_url).await?;

    // Run migrations
    account::infrastructure::db::run_migrations(&db_pool).await?;

    tracing::info!("Database migrations completed");

    // Start gRPC server
    let grpc_addr = config.grpc_addr.clone();
    tracing::info!("gRPC server listening on {}", grpc_addr);

    account::interface::grpc::start_server(grpc_addr, db_pool).await?;

    Ok(())
}
