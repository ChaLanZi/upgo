use crate::domain::risk::RiskEvent;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait RiskEventRepository: Send + Sync {
    async fn create(&self, event: &RiskEvent) -> Result<RiskEvent, sqlx::Error>;
    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<RiskEvent>, i64), sqlx::Error>;
}

pub struct PgRiskEventRepository {
    pool: PgPool,
}

impl PgRiskEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RiskEventRepository for PgRiskEventRepository {
    async fn create(&self, event: &RiskEvent) -> Result<RiskEvent, sqlx::Error> {
        let row = sqlx::query_as::<_, RiskEventRow>(
            r#"INSERT INTO risk_events (id, user_id, rule_name, condition, action, detail, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, user_id, rule_name, condition, action, detail, created_at"#
        )
        .bind(event.id)
        .bind(event.user_id)
        .bind(&event.rule_name)
        .bind(&event.condition)
        .bind(&event.action)
        .bind(&event.detail)
        .bind(event.created_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<RiskEvent>, i64), sqlx::Error> {
        let rows = sqlx::query_as::<_, RiskEventRow>(
            "SELECT id, user_id, rule_name, condition, action, detail, created_at FROM risk_events WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM risk_events WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let events: Vec<RiskEvent> = rows.into_iter().map(|r| r.into()).collect();
        Ok((events, count))
    }
}

#[derive(sqlx::FromRow)]
struct RiskEventRow {
    id: Uuid,
    user_id: Uuid,
    rule_name: String,
    condition: String,
    action: String,
    detail: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<RiskEventRow> for RiskEvent {
    fn from(r: RiskEventRow) -> Self {
        RiskEvent {
            id: r.id,
            user_id: r.user_id,
            rule_name: r.rule_name,
            condition: r.condition,
            action: r.action,
            detail: r.detail,
            created_at: r.created_at,
        }
    }
}
