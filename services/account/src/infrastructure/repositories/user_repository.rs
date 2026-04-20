use crate::domain::user::{AccountStatus, KycStatus, User, UserId};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create(&self, user: &User) -> Result<User, sqlx::Error>;
    async fn update(&self, user: &User) -> Result<User, sqlx::Error>;
    async fn exists_by_email(&self, email: &str) -> Result<bool, sqlx::Error>;
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, phone, nickname, password_hash, kyc_status, account_status, created_at, updated_at, version FROM users WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, phone, nickname, password_hash, kyc_status, account_status, created_at, updated_at, version FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, user: &User) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"INSERT INTO users (id, email, phone, nickname, password_hash, kyc_status, account_status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, email, phone, nickname, password_hash, kyc_status, account_status, created_at, updated_at, version"#
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.nickname)
        .bind(&user.password_hash)
        .bind(user.kyc_status.as_str())
        .bind(user.account_status.as_str())
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.version)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn update(&self, user: &User) -> Result<User, sqlx::Error> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"UPDATE users SET email=$2, phone=$3, nickname=$4, password_hash=$5,
             kyc_status=$6, account_status=$7, updated_at=$8, version=version+1
             WHERE id=$1 AND version=$9
             RETURNING id, email, phone, nickname, password_hash, kyc_status, account_status, created_at, updated_at, version"#
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.nickname)
        .bind(&user.password_hash)
        .bind(user.kyc_status.as_str())
        .bind(user.account_status.as_str())
        .bind(user.updated_at)
        .bind(user.version)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }
}

// Internal row type for sqlx queries
#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    phone: Option<String>,
    nickname: String,
    password_hash: String,
    kyc_status: String,
    account_status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    version: i32,
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        User {
            id: UserId(r.id),
            email: r.email,
            phone: r.phone,
            nickname: r.nickname,
            password_hash: r.password_hash,
            kyc_status: KycStatus::from_str(&r.kyc_status).unwrap_or(KycStatus::None),
            account_status: AccountStatus::from_str(&r.account_status)
                .unwrap_or(AccountStatus::PendingVerification),
            created_at: r.created_at,
            updated_at: r.updated_at,
            version: r.version,
        }
    }
}
