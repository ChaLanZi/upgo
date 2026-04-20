use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn store(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        token_hash: &str,
    ) -> Result<(), sqlx::Error>;
    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshTokenRow>, sqlx::Error>;
    async fn rotate(&self, old_hash: &str, new_hash: &str) -> Result<bool, sqlx::Error>;
    async fn blacklist(&self, token_hash: &str) -> Result<(), sqlx::Error>;
    async fn is_blacklisted(&self, token_hash: &str) -> Result<bool, sqlx::Error>;
    async fn delete_by_user(&self, user_id: Uuid) -> Result<u64, sqlx::Error>;
}

pub struct RefreshTokenRow {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub token_hash: String,
    pub blacklisted: bool,
}

pub struct PgRefreshTokenRepository {
    pool: PgPool,
}

impl PgRefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for PgRefreshTokenRepository {
    async fn store(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        token_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, session_id, token_hash, blacklisted) VALUES ($1, $2, $3, false)"
        )
        .bind(user_id)
        .bind(session_id)
        .bind(token_hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshTokenRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, RefreshTokenRowPg>(
            "SELECT user_id, session_id, token_hash, blacklisted FROM refresh_tokens WHERE token_hash = $1"
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| RefreshTokenRow {
            user_id: r.user_id,
            session_id: r.session_id,
            token_hash: r.token_hash,
            blacklisted: r.blacklisted,
        }))
    }

    async fn rotate(&self, old_hash: &str, new_hash: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE refresh_tokens SET token_hash = $2 WHERE token_hash = $1 AND blacklisted = false"
        )
        .bind(old_hash)
        .bind(new_hash)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn blacklist(&self, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE refresh_tokens SET blacklisted = true WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn is_blacklisted(&self, token_hash: &str) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM refresh_tokens WHERE token_hash = $1 AND blacklisted = true",
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct RefreshTokenRowPg {
    user_id: Uuid,
    session_id: Uuid,
    token_hash: String,
    blacklisted: bool,
}
