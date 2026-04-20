use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use tracing::info;

use crate::domain::events::AuthEvent;

/// Event publisher for publishing domain events to NATS.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &AuthEvent);
}

/// NATS-based event publisher.
pub struct NatsEventPublisher {
    nats: Arc<async_nats::Client>,
}

impl NatsEventPublisher {
    pub fn new(nats: Arc<async_nats::Client>) -> Self {
        Self { nats }
    }

    fn subject_for(event: &AuthEvent) -> &'static str {
        match event {
            AuthEvent::UserLoggedIn { .. } => "auth.user.logged_in",
            AuthEvent::UserRegistered { .. } => "auth.user.registered",
            AuthEvent::UserLoggedOut { .. } => "auth.user.logged_out",
            AuthEvent::PasswordChanged { .. } => "auth.user.password_changed",
            AuthEvent::EmailChanged { .. } => "auth.user.email_changed",
            AuthEvent::UserDeleted { .. } => "auth.user.deleted",
        }
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &AuthEvent) {
        let subject = Self::subject_for(event);
        let payload = serde_json::to_vec(event).unwrap_or_default();
        if let Err(e) = self.nats.publish(subject, payload.into()).await {
            tracing::warn!("Failed to publish event to NATS: {}", e);
        } else {
            info!("Published event to {}: {:?}", subject, event);
        }
    }
}

/// Log-only event publisher for development/testing (no NATS required).
pub struct LogEventPublisher;

#[async_trait]
impl EventPublisher for LogEventPublisher {
    async fn publish(&self, event: &AuthEvent) {
        info!("[EVENT] {:?}", event);
    }
}
