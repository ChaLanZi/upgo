use crate::domain::position::{Position, PositionHistory, PositionStatus};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait PositionRepository: Send + Sync {
    async fn find_by_user_and_symbol(
        &self,
        user_id: Uuid,
        symbol: &str,
    ) -> Result<Option<Position>, sqlx::Error>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Position>, sqlx::Error>;
    async fn find_open_by_user(&self, user_id: Uuid) -> Result<Vec<Position>, sqlx::Error>;
    async fn create(&self, position: &Position) -> Result<Position, sqlx::Error>;
    async fn update(&self, position: &Position) -> Result<Position, sqlx::Error>;
}

#[async_trait]
pub trait PositionHistoryRepository: Send + Sync {
    async fn create(&self, history: &PositionHistory) -> Result<PositionHistory, sqlx::Error>;
    async fn find_by_position(
        &self,
        position_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PositionHistory>, i64), sqlx::Error>;
}

pub struct PgPositionRepository {
    pool: PgPool,
}

impl PgPositionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PositionRepository for PgPositionRepository {
    async fn find_by_user_and_symbol(
        &self,
        user_id: Uuid,
        symbol: &str,
    ) -> Result<Option<Position>, sqlx::Error> {
        let row = sqlx::query_as::<_, PositionRow>(
            "SELECT id, user_id, symbol, quantity, cost_price, current_price, unrealized_pnl, status, created_at, updated_at, version FROM positions WHERE user_id = $1 AND symbol = $2 AND status = 'OPEN'"
        )
        .bind(user_id)
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Position>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PositionRow>(
            "SELECT id, user_id, symbol, quantity, cost_price, current_price, unrealized_pnl, status, created_at, updated_at, version FROM positions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_open_by_user(&self, user_id: Uuid) -> Result<Vec<Position>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PositionRow>(
            "SELECT id, user_id, symbol, quantity, cost_price, current_price, unrealized_pnl, status, created_at, updated_at, version FROM positions WHERE user_id = $1 AND status = 'OPEN' ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn create(&self, position: &Position) -> Result<Position, sqlx::Error> {
        let row = sqlx::query_as::<_, PositionRow>(
            r#"INSERT INTO positions (id, user_id, symbol, quantity, cost_price, current_price, unrealized_pnl, status, created_at, updated_at, version)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id, user_id, symbol, quantity, cost_price, current_price, unrealized_pnl, status, created_at, updated_at, version"#
        )
        .bind(position.id)
        .bind(position.user_id)
        .bind(&position.symbol)
        .bind(position.quantity)
        .bind(position.cost_price)
        .bind(position.current_price)
        .bind(position.unrealized_pnl())
        .bind(position.status.as_str())
        .bind(position.created_at)
        .bind(position.updated_at)
        .bind(position.version)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn update(&self, position: &Position) -> Result<Position, sqlx::Error> {
        let row = sqlx::query_as::<_, PositionRow>(
            r#"UPDATE positions SET quantity=$3, cost_price=$4, current_price=$5,
             unrealized_pnl=$6, status=$7, updated_at=$8, version=version+1
             WHERE id=$1 AND version=$9
             RETURNING id, user_id, symbol, quantity, cost_price, current_price, unrealized_pnl, status, created_at, updated_at, version"#
        )
        .bind(position.id)
        .bind(position.user_id)
        .bind(position.quantity)
        .bind(position.cost_price)
        .bind(position.current_price)
        .bind(position.unrealized_pnl())
        .bind(position.status.as_str())
        .bind(position.updated_at)
        .bind(position.version)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }
}

pub struct PgPositionHistoryRepository {
    pool: PgPool,
}

impl PgPositionHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PositionHistoryRepository for PgPositionHistoryRepository {
    async fn create(&self, history: &PositionHistory) -> Result<PositionHistory, sqlx::Error> {
        let row = sqlx::query_as::<_, PositionHistoryRow>(
            r#"INSERT INTO position_histories (id, position_id, user_id, symbol, change_type, change_quantity, quantity_after, price, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, position_id, user_id, symbol, change_type, change_quantity, quantity_after, price, created_at"#
        )
        .bind(history.id)
        .bind(history.position_id)
        .bind(history.user_id)
        .bind(&history.symbol)
        .bind(&history.change_type)
        .bind(history.change_quantity)
        .bind(history.quantity_after)
        .bind(history.price)
        .bind(history.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn find_by_position(
        &self,
        position_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<PositionHistory>, i64), sqlx::Error> {
        let rows = sqlx::query_as::<_, PositionHistoryRow>(
            "SELECT id, position_id, user_id, symbol, change_type, change_quantity, quantity_after, price, created_at FROM position_histories WHERE position_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(position_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM position_histories WHERE position_id = $1")
                .bind(position_id)
                .fetch_one(&self.pool)
                .await?;

        let entries: Vec<PositionHistory> = rows.into_iter().map(|r| r.into()).collect();
        Ok((entries, count))
    }
}

#[derive(sqlx::FromRow)]
struct PositionRow {
    id: Uuid,
    user_id: Uuid,
    symbol: String,
    quantity: i64,
    cost_price: i64,
    current_price: i64,
    #[allow(dead_code)]
    unrealized_pnl: i64,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    version: i32,
}

impl From<PositionRow> for Position {
    fn from(r: PositionRow) -> Self {
        Position {
            id: r.id,
            user_id: r.user_id,
            symbol: r.symbol,
            quantity: r.quantity,
            cost_price: r.cost_price,
            current_price: r.current_price,
            status: PositionStatus::from_str(&r.status).unwrap_or(PositionStatus::Open),
            created_at: r.created_at,
            updated_at: r.updated_at,
            version: r.version,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PositionHistoryRow {
    id: Uuid,
    position_id: Uuid,
    user_id: Uuid,
    symbol: String,
    change_type: String,
    change_quantity: i64,
    quantity_after: i64,
    price: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<PositionHistoryRow> for PositionHistory {
    fn from(r: PositionHistoryRow) -> Self {
        PositionHistory {
            id: r.id,
            position_id: r.position_id,
            user_id: r.user_id,
            symbol: r.symbol,
            change_type: r.change_type,
            change_quantity: r.change_quantity,
            quantity_after: r.quantity_after,
            price: r.price,
            created_at: r.created_at,
        }
    }
}
