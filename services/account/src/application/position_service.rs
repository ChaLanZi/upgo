use crate::domain::events::AccountEvent;
use crate::domain::position::{Position, PositionHistory};
use crate::infrastructure::nats::EventPublisher;
use crate::infrastructure::repositories::position_repository::{
    PositionHistoryRepository, PositionRepository,
};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

/// Application service for position-related use cases
pub struct PositionApplicationService {
    position_repo: Arc<dyn PositionRepository>,
    history_repo: Arc<dyn PositionHistoryRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl PositionApplicationService {
    pub fn new(
        position_repo: Arc<dyn PositionRepository>,
        history_repo: Arc<dyn PositionHistoryRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            position_repo,
            history_repo,
            event_publisher,
        }
    }

    /// Get all positions for a user
    pub async fn get_positions(&self, user_id: uuid::Uuid) -> Result<Vec<Position>> {
        self.position_repo
            .find_by_user(user_id)
            .await
            .map_err(|e| e.into())
    }

    /// Get a specific position for a user
    pub async fn get_position(&self, user_id: uuid::Uuid, symbol: &str) -> Result<Position> {
        self.position_repo
            .find_by_user_and_symbol(user_id, symbol)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Position not found"))
    }

    /// Get position change history
    pub async fn get_position_history(
        &self,
        position_id: uuid::Uuid,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<PositionHistory>, i64)> {
        let offset = ((page - 1).max(0) * page_size) as i64;
        self.history_repo
            .find_by_position(position_id, page_size as i64, offset)
            .await
            .map_err(|e| e.into())
    }

    /// Update position when a trade occurs
    pub async fn update_position(
        &self,
        user_id: uuid::Uuid,
        symbol: &str,
        quantity_change: i64,
        price: i64,
    ) -> Result<Position> {
        let position = self
            .position_repo
            .find_by_user_and_symbol(user_id, symbol)
            .await?;

        let (position, change_type) = if let Some(mut existing) = position {
            if quantity_change > 0 {
                existing.add(quantity_change, price);
                (existing, "BUY".to_string())
            } else {
                existing.reduce(-quantity_change, price)?;
                (existing, "SELL".to_string())
            }
        } else {
            if quantity_change <= 0 {
                anyhow::bail!("Cannot sell: no open position for symbol {}", symbol);
            }
            (
                Position::new(user_id, symbol, quantity_change, price),
                "BUY".to_string(),
            )
        };

        let saved = if position.status.to_string() == "CLOSED" || position.quantity == 0 {
            self.position_repo.update(&position).await?
        } else {
            self.position_repo.create(&position).await?
        };

        let history = PositionHistory::new(
            saved.id,
            user_id,
            symbol,
            &change_type,
            quantity_change.abs(),
            saved.quantity,
            price,
        );
        self.history_repo.create(&history).await?;

        self.event_publisher
            .publish(&AccountEvent::PositionChanged {
                user_id,
                position_id: saved.id,
                symbol: symbol.to_string(),
                change_type,
                change_quantity: quantity_change.abs(),
                quantity_after: saved.quantity,
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved)
    }
}
