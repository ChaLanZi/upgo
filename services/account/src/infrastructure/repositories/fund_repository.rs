use crate::domain::fund::{Currency, FundAccount, FundTransaction, TransactionType};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait FundAccountRepository: Send + Sync {
    async fn find_by_user_and_currency(
        &self,
        user_id: Uuid,
        currency: &str,
    ) -> Result<Option<FundAccount>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<FundAccount>, sqlx::Error>;
    async fn create(&self, account: &FundAccount) -> Result<FundAccount, sqlx::Error>;
    async fn update_balance(&self, account: &FundAccount) -> Result<FundAccount, sqlx::Error>;
}

#[async_trait]
pub trait FundTransactionRepository: Send + Sync {
    async fn create(&self, tx: &FundTransaction) -> Result<FundTransaction, sqlx::Error>;
    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FundTransaction>, i64), sqlx::Error>;
}

pub struct PgFundAccountRepository {
    pool: PgPool,
}

impl PgFundAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FundAccountRepository for PgFundAccountRepository {
    async fn find_by_user_and_currency(
        &self,
        user_id: Uuid,
        currency: &str,
    ) -> Result<Option<FundAccount>, sqlx::Error> {
        let row = sqlx::query_as::<_, FundAccountRow>(
            "SELECT id, user_id, currency, balance, frozen_balance, version FROM fund_accounts WHERE user_id = $1 AND currency = $2"
        )
        .bind(user_id)
        .bind(currency)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FundAccount>, sqlx::Error> {
        let row = sqlx::query_as::<_, FundAccountRow>(
            "SELECT id, user_id, currency, balance, frozen_balance, version FROM fund_accounts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn create(&self, account: &FundAccount) -> Result<FundAccount, sqlx::Error> {
        let row = sqlx::query_as::<_, FundAccountRow>(
            r#"INSERT INTO fund_accounts (id, user_id, currency, balance, frozen_balance, version)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, user_id, currency, balance, frozen_balance, version"#
        )
        .bind(account.id)
        .bind(account.user_id)
        .bind(account.currency.as_str())
        .bind(account.balance)
        .bind(account.frozen_balance)
        .bind(account.version)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn update_balance(&self, account: &FundAccount) -> Result<FundAccount, sqlx::Error> {
        let row = sqlx::query_as::<_, FundAccountRow>(
            r#"UPDATE fund_accounts SET balance=$3, frozen_balance=$4, version=version+1
             WHERE id=$1 AND version=$5
             RETURNING id, user_id, currency, balance, frozen_balance, version"#
        )
        .bind(account.id)
        .bind(account.user_id)
        .bind(account.balance)
        .bind(account.frozen_balance)
        .bind(account.version)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }
}

pub struct PgFundTransactionRepository {
    pool: PgPool,
}

impl PgFundTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FundTransactionRepository for PgFundTransactionRepository {
    async fn create(&self, tx: &FundTransaction) -> Result<FundTransaction, sqlx::Error> {
        let row = sqlx::query_as::<_, FundTransactionRow>(
            r#"INSERT INTO fund_transactions (id, user_id, account_id, type, amount, balance_before, balance_after, order_id, remark, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, user_id, account_id, type, amount, balance_before, balance_after, order_id, remark, created_at"#
        )
        .bind(tx.id)
        .bind(tx.user_id)
        .bind(tx.account_id)
        .bind(tx.transaction_type.as_str())
        .bind(tx.amount)
        .bind(tx.balance_before)
        .bind(tx.balance_after)
        .bind(tx.order_id.as_deref())
        .bind(&tx.remark)
        .bind(tx.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FundTransaction>, i64), sqlx::Error> {
        let rows = sqlx::query_as::<_, FundTransactionRow>(
            "SELECT id, user_id, account_id, type, amount, balance_before, balance_after, order_id, remark, created_at FROM fund_transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM fund_transactions WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let transactions: Vec<FundTransaction> = rows.into_iter().map(|r| r.into()).collect();
        Ok((transactions, count))
    }
}

#[derive(sqlx::FromRow)]
struct FundAccountRow {
    id: Uuid,
    user_id: Uuid,
    currency: String,
    balance: i64,
    frozen_balance: i64,
    version: i32,
}

impl From<FundAccountRow> for FundAccount {
    fn from(r: FundAccountRow) -> Self {
        FundAccount {
            id: r.id,
            user_id: r.user_id,
            currency: Currency::from_str(&r.currency).unwrap_or(Currency::CNY),
            balance: r.balance,
            frozen_balance: r.frozen_balance,
            version: r.version,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FundTransactionRow {
    id: Uuid,
    user_id: Uuid,
    account_id: Uuid,
    #[sqlx(rename = "type")]
    tx_type: String,
    amount: i64,
    balance_before: i64,
    balance_after: i64,
    order_id: Option<String>,
    remark: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<FundTransactionRow> for FundTransaction {
    fn from(r: FundTransactionRow) -> Self {
        FundTransaction {
            id: r.id,
            user_id: r.user_id,
            account_id: r.account_id,
            transaction_type: match r.tx_type.as_str() {
                "DEPOSIT" => TransactionType::Deposit,
                "WITHDRAWAL" => TransactionType::Withdrawal,
                "FUND_FROZEN" => TransactionType::FundFrozen,
                "FUND_UNFROZEN" => TransactionType::FundUnfrozen,
                _ => TransactionType::Deposit,
            },
            amount: r.amount,
            balance_before: r.balance_before,
            balance_after: r.balance_after,
            order_id: r.order_id,
            remark: r.remark,
            created_at: r.created_at,
        }
    }
}
