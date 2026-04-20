use crate::domain::session::AuthSession;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthSession>, sqlx::Error>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuthSession>, sqlx::Error>;
    async fn find_by_user_and_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> Result<Vec<AuthSession>, sqlx::Error>;
    async fn create(&self, session: &AuthSession) -> Result<AuthSession, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error>;
    async fn delete_by_user(&self, user_id: Uuid) -> Result<u64, sqlx::Error>;
    async fn delete_by_user_except(
        &self,
        user_id: Uuid,
        exclude_session_id: Uuid,
    ) -> Result<u64, sqlx::Error>;
    async fn count_by_user_and_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> Result<i64, sqlx::Error>;
}

pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthSession>, sqlx::Error> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, user_id, platform, refresh_token_hash, created_at, expires_at, last_active_at FROM sessions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<AuthSession>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, user_id, platform, refresh_token_hash, created_at, expires_at, last_active_at FROM sessions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_user_and_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> Result<Vec<AuthSession>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SessionRow>(
            "SELECT id, user_id, platform, refresh_token_hash, created_at, expires_at, last_active_at FROM sessions WHERE user_id = $1 AND platform = $2 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .bind(platform)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn create(&self, session: &AuthSession) -> Result<AuthSession, sqlx::Error> {
        let row = sqlx::query_as::<_, SessionRow>(
            r#"INSERT INTO sessions (id, user_id, platform, refresh_token_hash, created_at, expires_at, last_active_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, user_id, platform, refresh_token_hash, created_at, expires_at, last_active_at"#
        )
        .bind(session.id)
        .bind(session.user_id)
        .bind(session.platform.as_str())
        .bind(&session.refresh_token_hash)
        .bind(session.created_at)
        .bind(session.expires_at)
        .bind(session.last_active_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn delete_by_user_except(
        &self,
        user_id: Uuid,
        exclude_session_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id != $2")
            .bind(user_id)
            .bind(exclude_session_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn count_by_user_and_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND platform = $2")
                .bind(user_id)
                .bind(platform)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }
}

#[derive(sqlx::FromRow)]
struct SessionRow {
    id: Uuid,
    user_id: Uuid,
    platform: String,
    refresh_token_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    last_active_at: chrono::DateTime<chrono::Utc>,
}

impl From<SessionRow> for AuthSession {
    fn from(r: SessionRow) -> Self {
        AuthSession {
            id: r.id,
            user_id: r.user_id,
            platform: crate::domain::session::Platform::from_str(&r.platform)
                .unwrap_or(crate::domain::session::Platform::Web),
            refresh_token_hash: r.refresh_token_hash,
            created_at: r.created_at,
            expires_at: r.expires_at,
            last_active_at: r.last_active_at,
        }
    }
}
