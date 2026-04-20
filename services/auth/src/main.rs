#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut,)
)]

use std::sync::Arc;

use anyhow::Result;
use auth::application::auth_service::AuthApplicationService;
use auth::infrastructure::account_client::{DevAccountClient, GrpcAccountClient};
use auth::infrastructure::cleanup_task::AccountDeletionCleanupTask;
use auth::infrastructure::config::AppConfig;
use auth::infrastructure::event_publisher::{
    EventPublisher, LogEventPublisher, NatsEventPublisher,
};
use auth::infrastructure::jwt_service::JwtService;
use auth::infrastructure::mail_service::MailService;
use auth::infrastructure::repositories::account_deletion_repository::PgAccountDeletionRepository;
use auth::infrastructure::repositories::email_verification_repository::PgEmailVerificationRepository;
use auth::infrastructure::repositories::refresh_token_repository::PgRefreshTokenRepository;
use auth::infrastructure::repositories::session_repository::PgSessionRepository;
use auth::interface::grpc::AuthGrpcHandler;
use contracts::proto::auth::auth_service_server::AuthServiceServer;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    tracing::info!("Starting Auth Service...");

    // ── Database connection ────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("../../migrations/auth").run(&pool).await?;
    tracing::info!("Database migrations applied");

    // ── Infrastructure ─────────────────────────────────────
    let jwt_service = Arc::new(JwtService::new(&config.jwt_secret));
    let mail_service = Arc::new(MailService::new(
        &config.smtp_host,
        config.smtp_port,
        &config.smtp_username,
        &config.smtp_password,
        &config.smtp_from,
    ));

    // Repositories
    let session_repo = Arc::new(PgSessionRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PgRefreshTokenRepository::new(pool.clone()));
    let email_verification_repo = Arc::new(PgEmailVerificationRepository::new(pool.clone()));
    let account_deletion_repo = Arc::new(PgAccountDeletionRepository::new(pool.clone()));

    // Account service client: use DevAccountClient for now (no running account service)
    // Switch to GrpcAccountClient when account service is deployed:
    //   let account_client = Arc::new(GrpcAccountClient::new("http://account:50051"));
    let account_client: Arc<dyn auth::infrastructure::account_client::AccountClient> =
        Arc::new(DevAccountClient);

    // Event publisher: use LogEventPublisher for dev, NatsEventPublisher for production
    let event_publisher: Arc<dyn EventPublisher> = if cfg!(feature = "nats") {
        let nats_client = async_nats::connect(&config.nats_url).await?;
        Arc::new(NatsEventPublisher::new(Arc::new(nats_client)))
    } else {
        Arc::new(LogEventPublisher)
    };

    // ── Application Service ────────────────────────────────
    let app_service = Arc::new(AuthApplicationService::new(
        session_repo.clone(),
        refresh_token_repo.clone(),
        email_verification_repo.clone(),
        account_deletion_repo.clone(),
        account_client.clone(),
        event_publisher.clone(),
        jwt_service.clone(),
        mail_service.clone(),
    ));

    // ── Background Tasks ───────────────────────────────────
    // Account deletion cleanup (runs every hour)
    let cleanup_task = AccountDeletionCleanupTask::new(
        pool.clone(),
        account_client.clone(),
        event_publisher.clone(),
    );
    tokio::spawn(async move {
        cleanup_task.run().await;
    });

    // ── gRPC Handler ───────────────────────────────────────
    let auth_handler = AuthGrpcHandler::new(app_service);
    let auth_service = AuthServiceServer::new(auth_handler);

    // ── Health check endpoint (axum on a separate port) ────
    let health_svc = axum::Router::new().route("/health", axum::routing::get(|| async { "OK" }));

    let grpc_addr = config.grpc_addr.clone();
    let health_addr = "0.0.0.0:9090".to_string(); // health check on port 9090

    tracing::info!("gRPC server listening on {}", grpc_addr);
    tracing::info!("Health endpoint listening on {}", health_addr);

    // ── Start servers ──────────────────────────────────────
    let health_listener = tokio::net::TcpListener::bind(&health_addr).await?;

    // Health check server in background
    tokio::spawn(async move {
        if let Err(e) = axum::serve(health_listener, health_svc).await {
            tracing::error!("Health server error: {}", e);
        }
    });

    let mut health = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    // gRPC server (main task)
    tokio::select! {
        result = Server::builder()
            .add_service(auth_service)
            .serve(grpc_addr.parse()?) => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        _ = health.recv() => {
            tracing::info!("Received SIGTERM, shutting down...");
        }
    }

    Ok(())
}
