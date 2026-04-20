use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use tracing::info;

use crate::domain::events::AuthEvent;
use crate::infrastructure::account_client::AccountClient;
use crate::infrastructure::event_publisher::EventPublisher;

/// Background task that periodically checks for account deletions
/// whose cooldown period has expired and performs permanent cleanup.
pub struct AccountDeletionCleanupTask {
    pool: PgPool,
    account_client: Arc<dyn AccountClient>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl AccountDeletionCleanupTask {
    pub fn new(
        pool: PgPool,
        account_client: Arc<dyn AccountClient>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            pool,
            account_client,
            event_publisher,
        }
    }

    /// Start the background cleanup loop. Runs every hour.
    pub async fn run(self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = self.do_cleanup().await {
                tracing::warn!("Account deletion cleanup error: {}", e);
            }
        }
    }

    async fn do_cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();

        // Find all deletions that are ready for permanent deletion
        let rows = sqlx::query_as::<_, CleanupRow>(
            r#"SELECT user_id, soft_deleted_at, permanent_delete_at
             FROM account_deletions
             WHERE permanent_delete_at <= $1
               AND cancelled = false
               AND soft_deleted_at IS NOT NULL"#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        for row in &rows {
            info!(
                "Permanently deleting account {} (soft-deleted at: {:?})",
                row.user_id, row.soft_deleted_at
            );

            // 1. Notify account service to clean up user data
            if let Err(e) = self
                .account_client
                .soft_delete_user(row.user_id, &now.to_rfc3339())
                .await
            {
                tracing::warn!(
                    "Failed to notify account service for user {}: {}",
                    row.user_id,
                    e
                );
                // Continue with local cleanup anyway
            }

            // 2. Clean up sessions
            sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                .bind(row.user_id)
                .execute(&self.pool)
                .await?;

            // 3. Clean up refresh tokens
            sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
                .bind(row.user_id)
                .execute(&self.pool)
                .await?;

            // 4. Remove the deletion record
            sqlx::query("DELETE FROM account_deletions WHERE user_id = $1")
                .bind(row.user_id)
                .execute(&self.pool)
                .await?;

            // 5. Publish event
            self.event_publisher
                .publish(&AuthEvent::UserDeleted {
                    user_id: row.user_id,
                    soft_deleted_at: row.soft_deleted_at.unwrap_or(now),
                    permanent_delete_at: now,
                    timestamp: now,
                })
                .await;

            info!("Account {} permanently deleted", row.user_id);
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct CleanupRow {
    user_id: uuid::Uuid,
    soft_deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    permanent_delete_at: Option<chrono::DateTime<chrono::Utc>>,
}
