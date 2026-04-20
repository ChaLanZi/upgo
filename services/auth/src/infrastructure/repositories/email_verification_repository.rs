use crate::domain::email_verification::{EmailVerification, VerificationPurpose};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait EmailVerificationRepository: Send + Sync {
    async fn create(&self, verification: &EmailVerification) -> Result<(), sqlx::Error>;
    async fn find_by_email_and_purpose(
        &self,
        email: &str,
        purpose: &str,
    ) -> Result<Option<EmailVerification>, sqlx::Error>;
    async fn delete_by_email(&self, email: &str) -> Result<u64, sqlx::Error>;
    async fn cleanup_expired(&self) -> Result<u64, sqlx::Error>;
}

pub struct PgEmailVerificationRepository {
    pool: PgPool,
}

impl PgEmailVerificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailVerificationRepository for PgEmailVerificationRepository {
    async fn create(&self, verification: &EmailVerification) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO email_verifications (email, code, purpose, created_at, expires_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&verification.email)
        .bind(&verification.code)
        .bind(match verification.purpose {
            VerificationPurpose::Registration => "registration",
            VerificationPurpose::EmailChange => "email_change",
            VerificationPurpose::AccountDeletion => "account_deletion",
        })
        .bind(verification.created_at)
        .bind(verification.expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_email_and_purpose(
        &self,
        email: &str,
        purpose: &str,
    ) -> Result<Option<EmailVerification>, sqlx::Error> {
        let row = sqlx::query_as::<_, EmailVerificationRow>(
            "SELECT email, code, purpose, created_at, expires_at FROM email_verifications WHERE email = $1 AND purpose = $2 ORDER BY created_at DESC LIMIT 1"
        )
        .bind(email)
        .bind(purpose)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn delete_by_email(&self, email: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM email_verifications WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM email_verifications WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct EmailVerificationRow {
    email: String,
    code: String,
    purpose: String,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<EmailVerificationRow> for EmailVerification {
    fn from(r: EmailVerificationRow) -> Self {
        let purpose = match r.purpose.as_str() {
            "email_change" => VerificationPurpose::EmailChange,
            "account_deletion" => VerificationPurpose::AccountDeletion,
            _ => VerificationPurpose::Registration,
        };
        Self {
            email: r.email,
            code: r.code,
            purpose,
            created_at: r.created_at,
            expires_at: r.expires_at,
        }
    }
}
