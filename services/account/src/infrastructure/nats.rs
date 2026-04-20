use crate::domain::events::AccountEvent;
use anyhow::Result;
use async_trait::async_trait;

/// Event publisher trait for domain events
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &AccountEvent) -> Result<()>;
}

/// NATS event publisher implementation
pub struct NatsEventPublisher {
    nc: async_nats::Client,
}

impl NatsEventPublisher {
    pub fn new(nc: async_nats::Client) -> Self {
        Self { nc }
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &AccountEvent) -> Result<()> {
        let (subject, payload) = match event {
            AccountEvent::FundChanged { user_id, .. } => (
                format!("account.fund.changed.{}", user_id),
                serde_json::to_string(event)?,
            ),
            AccountEvent::PositionChanged { user_id, .. } => (
                format!("account.position.changed.{}", user_id),
                serde_json::to_string(event)?,
            ),
            AccountEvent::RiskEventTriggered { user_id, .. } => (
                format!("account.risk.triggered.{}", user_id),
                serde_json::to_string(event)?,
            ),
            AccountEvent::MarginWarning { user_id, .. } => (
                format!("account.risk.margin_warning.{}", user_id),
                serde_json::to_string(event)?,
            ),
            AccountEvent::UserRegistered { user_id, .. } => (
                format!("account.user.registered.{}", user_id),
                serde_json::to_string(event)?,
            ),
            AccountEvent::KycStatusChanged { user_id, .. } => (
                format!("account.user.kyc.{}", user_id),
                serde_json::to_string(event)?,
            ),
        };

        self.nc
            .publish(subject, payload.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish NATS event: {}", e))?;

        Ok(())
    }
}

/// No-op event publisher for testing
pub struct NoopEventPublisher;

#[async_trait]
impl EventPublisher for NoopEventPublisher {
    async fn publish(&self, _event: &AccountEvent) -> Result<()> {
        Ok(())
    }
}
