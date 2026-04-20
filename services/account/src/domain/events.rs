use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Domain events emitted by the account service
#[derive(Debug, Clone, Serialize)]
pub enum AccountEvent {
    /// User registered
    UserRegistered {
        user_id: Uuid,
        email: String,
        timestamp: DateTime<Utc>,
    },
    /// KYC status changed
    KycStatusChanged {
        user_id: Uuid,
        old_status: String,
        new_status: String,
        timestamp: DateTime<Utc>,
    },
    /// Funds changed (deposit, withdrawal)
    FundChanged {
        user_id: Uuid,
        account_id: Uuid,
        transaction_type: String,
        amount: i64,
        balance_after: i64,
        timestamp: DateTime<Utc>,
    },
    /// Position changed (created, updated, closed)
    PositionChanged {
        user_id: Uuid,
        position_id: Uuid,
        symbol: String,
        change_type: String,
        change_quantity: i64,
        quantity_after: i64,
        timestamp: DateTime<Utc>,
    },
    /// Risk event triggered
    RiskEventTriggered {
        user_id: Uuid,
        rule_name: String,
        action: String,
        detail: String,
        timestamp: DateTime<Utc>,
    },
    /// Margin warning
    MarginWarning {
        user_id: Uuid,
        margin_ratio: f64,
        level: String,
        timestamp: DateTime<Utc>,
    },
}
