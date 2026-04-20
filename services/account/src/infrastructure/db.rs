use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Initialize the database connection pool
pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

/// Run database migrations using SQL files
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Use sqlx migrations from the migrations/account directory
    sqlx::migrate!("../../migrations/account")
        .run(pool)
        .await
        .context("Failed to run database migrations")?;

    Ok(())
}
