use anyhow::Result;
use sqlx::PgPool;

pub mod user_handler;
pub mod fund_handler;
pub mod position_handler;
pub mod risk_handler;

pub use user_handler::*;
pub use fund_handler::*;
pub use position_handler::*;
pub use risk_handler::*;

/// Start the gRPC server
pub async fn start_server(addr: String, _pool: PgPool) -> Result<()> {
    tracing::info!("gRPC server starting on {}", addr);

    // In a full implementation, this would use tonic::transport::Server
    // with the generated proto service implementations.

    tracing::info!("Account service gRPC server ready");

    // Keep alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
