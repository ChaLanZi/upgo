use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Risk control rule configuration
#[derive(Debug, Clone)]
pub struct RiskRule {
    pub name: String,
    pub rule_type: RiskRuleType,
    pub threshold: f64,
    pub action: RiskAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskRuleType {
    PositionLimit,         // Max shares per symbol
    IndustryConcentration, // Max % in single industry
    MarginRatio,           // Min margin ratio
    LeverageRatio,         // Max leverage
}

impl std::fmt::Display for RiskRuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskRuleType::PositionLimit => write!(f, "PositionLimit"),
            RiskRuleType::IndustryConcentration => write!(f, "IndustryConcentration"),
            RiskRuleType::MarginRatio => write!(f, "MarginRatio"),
            RiskRuleType::LeverageRatio => write!(f, "LeverageRatio"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskAction {
    Warn,
    Reject,
    Liquidate,
}

impl std::fmt::Display for RiskAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskAction::Warn => write!(f, "Warn"),
            RiskAction::Reject => write!(f, "Reject"),
            RiskAction::Liquidate => write!(f, "Liquidate"),
        }
    }
}

/// Risk event record
#[derive(Debug, Clone)]
pub struct RiskEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub rule_name: String,
    pub condition: String,
    pub action: String,
    pub detail: String,
    pub created_at: DateTime<Utc>,
}

impl RiskEvent {
    pub fn new(
        user_id: Uuid,
        rule_name: &str,
        condition: &str,
        action: &str,
        detail: &str,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id,
            rule_name: rule_name.to_string(),
            condition: condition.to_string(),
            action: action.to_string(),
            detail: detail.to_string(),
            created_at: Utc::now(),
        }
    }
}

/// Result of a risk check
#[derive(Debug, Clone)]
pub struct RiskCheckResult {
    pub allowed: bool,
    pub warnings: Vec<String>,
    pub reject_reason: Option<String>,
}

impl RiskCheckResult {
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            warnings: vec![],
            reject_reason: None,
        }
    }

    pub fn with_warning(warning: &str) -> Self {
        Self {
            allowed: true,
            warnings: vec![warning.to_string()],
            reject_reason: None,
        }
    }

    pub fn rejected(reason: &str) -> Self {
        Self {
            allowed: false,
            warnings: vec![],
            reject_reason: Some(reason.to_string()),
        }
    }
}

/// Margin level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarginLevel {
    Safe,
    Warning,
    Danger,
    Liquidation,
}

impl std::fmt::Display for MarginLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl MarginLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarginLevel::Safe => "SAFE",
            MarginLevel::Warning => "WARNING",
            MarginLevel::Danger => "DANGER",
            MarginLevel::Liquidation => "LIQUIDATION",
        }
    }
}

/// Margin information
#[derive(Debug, Clone)]
pub struct MarginInfo {
    pub margin_ratio: f64,
    pub warning_threshold: f64,
    pub liquidation_threshold: f64,
    pub level: MarginLevel,
}
