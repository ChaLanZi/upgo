use crate::domain::events::AccountEvent;
use crate::domain::risk::{MarginInfo, RiskCheckResult, RiskEvent, RiskRule};
use crate::domain::services::RiskControlService;
use crate::infrastructure::nats::EventPublisher;
use crate::infrastructure::repositories::position_repository::PositionRepository;
use crate::infrastructure::repositories::risk_repository::RiskEventRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

/// Application service for risk-related use cases
pub struct RiskApplicationService {
    position_repo: Arc<dyn PositionRepository>,
    risk_event_repo: Arc<dyn RiskEventRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    risk_rules: Vec<RiskRule>,
}

impl RiskApplicationService {
    pub fn new(
        position_repo: Arc<dyn PositionRepository>,
        risk_event_repo: Arc<dyn RiskEventRepository>,
        event_publisher: Arc<dyn EventPublisher>,
        risk_rules: Vec<RiskRule>,
    ) -> Self {
        Self {
            position_repo,
            risk_event_repo,
            event_publisher,
            risk_rules,
        }
    }

    /// Check if an order passes all risk rules
    pub async fn check_order_risk(
        &self,
        user_id: uuid::Uuid,
        symbol: &str,
        _direction: &str,
        order_quantity: i64,
        order_price: i64,
    ) -> Result<RiskCheckResult> {
        let positions = self.position_repo.find_open_by_user(user_id).await?;
        let total_balance = 0; // TODO: fetch from fund service

        let result = RiskControlService::check_order_risk(
            &positions,
            &self.risk_rules,
            symbol,
            order_quantity,
            order_price,
            total_balance,
        );

        // Log risk events
        if !result.allowed {
            let event = RiskEvent::new(
                user_id,
                "OrderRiskCheck",
                &format!(
                    "symbol={}, qty={}, price={}",
                    symbol, order_quantity, order_price
                ),
                "REJECT",
                result.reject_reason.as_deref().unwrap_or("Unknown"),
            );
            self.risk_event_repo.create(&event).await?;

            self.event_publisher
                .publish(&AccountEvent::RiskEventTriggered {
                    user_id,
                    rule_name: "OrderRiskCheck".to_string(),
                    action: "REJECT".to_string(),
                    detail: result.reject_reason.clone().unwrap_or_default(),
                    timestamp: Utc::now(),
                })
                .await?;
        }

        Ok(result)
    }

    /// Get margin ratio for a user
    pub async fn get_margin_ratio(&self, user_id: uuid::Uuid) -> Result<MarginInfo> {
        let positions = self.position_repo.find_open_by_user(user_id).await?;
        let total_position_value: i64 = positions.iter().map(|p| p.market_value()).sum();
        let total_balance = 0; // TODO: fetch from fund service

        let warning_threshold = self
            .risk_rules
            .iter()
            .find(|r| r.rule_type.to_string() == "MarginRatio")
            .map(|r| r.threshold)
            .unwrap_or(1.5);

        let liquidation_threshold = warning_threshold * 0.73; // ~1.1 if warning is 1.5

        let info = RiskControlService::calculate_margin_ratio(
            total_position_value,
            total_balance,
            warning_threshold,
            liquidation_threshold,
        );

        // Publish warning if needed
        if info.level.to_string() == "WARNING" || info.level.to_string() == "DANGER" {
            self.event_publisher
                .publish(&AccountEvent::MarginWarning {
                    user_id,
                    margin_ratio: info.margin_ratio,
                    level: info.level.as_str().to_string(),
                    timestamp: Utc::now(),
                })
                .await?;
        }

        Ok(info)
    }

    /// Get risk events for a user
    pub async fn get_risk_events(
        &self,
        user_id: uuid::Uuid,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<RiskEvent>, i64)> {
        let offset = ((page - 1).max(0) * page_size) as i64;
        self.risk_event_repo
            .find_by_user(user_id, page_size as i64, offset)
            .await
            .map_err(|e| e.into())
    }

    /// Reload risk rules (called on config change)
    pub fn reload_rules(&mut self, new_rules: Vec<RiskRule>) {
        self.risk_rules = new_rules;
    }
}
