use crate::domain::account_deletion::AccountDeletion;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait AccountDeletionRepository: Send + Sync {
    async fn create(&self, deletion: &AccountDeletion) -> Result<(), sqlx::Error>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<AccountDeletion>, sqlx::Error>;
    async fn update(&self, deletion: &AccountDeletion) -> Result<(), sqlx::Error>;
    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<u64, sqlx::Error>;
}

pub struct PgAccountDeletionRepository {
    pool: PgPool,
}

impl PgAccountDeletionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountDeletionRepository for PgAccountDeletionRepository {
    async fn create(&self, deletion: &AccountDeletion) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO account_deletions (user_id, confirmation_code, requested_at, soft_deleted_at, permanent_delete_at, cancelled)
             VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(deletion.user_id)
        .bind(&deletion.confirmation_code)
        .bind(deletion.requested_at)
        .bind(deletion.soft_deleted_at)
        .bind(deletion.permanent_delete_at)
        .bind(deletion.cancelled)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<AccountDeletion>, sqlx::Error> {
        let row = sqlx::query_as::<_, AccountDeletionRow>(
            r#"SELECT user_id, confirmation_code, requested_at, soft_deleted_at, permanent_delete_at, cancelled
             FROM account_deletions WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn update(&self, deletion: &AccountDeletion) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE account_deletions
             SET confirmation_code = $2, soft_deleted_at = $3, permanent_delete_at = $4, cancelled = $5
             WHERE user_id = $1"#,
        )
        .bind(deletion.user_id)
        .bind(&deletion.confirmation_code)
        .bind(deletion.soft_deleted_at)
        .bind(deletion.permanent_delete_at)
        .bind(deletion.cancelled)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM account_deletions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct AccountDeletionRow {
    user_id: Uuid,
    confirmation_code: String,
    requested_at: chrono::DateTime<chrono::Utc>,
    soft_deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    permanent_delete_at: Option<chrono::DateTime<chrono::Utc>>,
    cancelled: bool,
}

impl From<AccountDeletionRow> for AccountDeletion {
    fn from(r: AccountDeletionRow) -> Self {
        let mut d = AccountDeletion::new(r.user_id, r.confirmation_code);
        d.requested_at = r.requested_at;
        d.soft_deleted_at = r.soft_deleted_at;
        d.permanent_delete_at = r.permanent_delete_at;
        d.cancelled = r.cancelled;
        d
    }
}
