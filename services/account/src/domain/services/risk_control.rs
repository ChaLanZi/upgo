use crate::domain::position::Position;
use crate::domain::risk::{MarginInfo, MarginLevel, RiskCheckResult, RiskRule, RiskRuleType};

/// Risk control domain service
pub struct RiskControlService;

impl RiskControlService {
    /// Check if an order passes all risk rules
    pub fn check_order_risk(
        positions: &[Position],
        rules: &[RiskRule],
        symbol: &str,
        order_quantity: i64,
        order_price: i64,
        total_balance: i64,
    ) -> RiskCheckResult {
        let mut warnings = Vec::new();

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            match rule.rule_type {
                RiskRuleType::PositionLimit => {
                    let current_qty = positions
                        .iter()
                        .find(|p| p.symbol == symbol && p.status.to_string() == "OPEN")
                        .map(|p| p.quantity)
                        .unwrap_or(0);

                    if current_qty + order_quantity > rule.threshold as i64 {
                        return RiskCheckResult::rejected(&format!(
                            "Position limit exceeded: max {} shares for {}",
                            rule.threshold as i64, symbol
                        ));
                    }
                }
                RiskRuleType::LeverageRatio => {
                    let total_position_value: i64 =
                        positions.iter().map(|p| p.market_value()).sum();
                    let new_order_value = order_quantity * order_price;
                    let total_after = total_position_value + new_order_value;

                    // Leverage = total_position_value / net_assets
                    let net_assets = total_balance;
                    if net_assets > 0 {
                        let leverage = total_after as f64 / net_assets as f64;
                        if leverage > rule.threshold {
                            return RiskCheckResult::rejected(&format!(
                                "Leverage limit exceeded: {:.2}x (max {:.1}x)",
                                leverage, rule.threshold
                            ));
                        }
                        if leverage > rule.threshold * 0.8 {
                            warnings.push(format!(
                                "Leverage approaching limit: {:.2}x (limit {:.1}x)",
                                leverage, rule.threshold
                            ));
                        }
                    }
                }
                RiskRuleType::IndustryConcentration => {
                    // Simplified: skip detailed industry check without market data
                    // In production, this would look up the symbol's industry
                    warnings.push(
                        "Industry concentration check skipped: requires market data".to_string(),
                    );
                }
                RiskRuleType::MarginRatio => {
                    // Checked separately via get_margin_ratio
                }
            }
        }

        if warnings.is_empty() {
            RiskCheckResult::allowed()
        } else {
            RiskCheckResult {
                allowed: true,
                warnings,
                reject_reason: None,
            }
        }
    }

    /// Calculate margin ratio
    pub fn calculate_margin_ratio(
        total_position_value: i64,
        total_balance: i64,
        warning_threshold: f64,
        liquidation_threshold: f64,
    ) -> MarginInfo {
        let margin_ratio = if total_position_value > 0 {
            total_balance as f64 / total_position_value as f64
        } else {
            1.0 // No positions, margin ratio is 1.0 (safe)
        };

        let level = if margin_ratio <= liquidation_threshold {
            MarginLevel::Liquidation
        } else if margin_ratio <= warning_threshold {
            MarginLevel::Danger
        } else if margin_ratio <= warning_threshold * 1.2 {
            MarginLevel::Warning
        } else {
            MarginLevel::Safe
        };

        MarginInfo {
            margin_ratio,
            warning_threshold,
            liquidation_threshold,
            level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::position::Position;
    use crate::domain::risk::{RiskAction, RiskRule, RiskRuleType};
    use uuid::Uuid;

    fn make_position(symbol: &str, quantity: i64, price: i64) -> Position {
        Position::new(Uuid::now_v7(), symbol, quantity, price)
    }

    fn make_rule(rule_type: RiskRuleType, threshold: f64, action: RiskAction) -> RiskRule {
        RiskRule {
            name: format!("{:?}", rule_type),
            rule_type,
            threshold,
            action,
            enabled: true,
        }
    }

    #[test]
    fn test_position_limit_allowed() {
        let positions = vec![make_position("AAPL", 50, 5000)];
        let rules = vec![make_rule(
            RiskRuleType::PositionLimit,
            100.0,
            RiskAction::Reject,
        )];

        let result =
            RiskControlService::check_order_risk(&positions, &rules, "AAPL", 30, 5000, 1_000_000);
        assert!(result.allowed);
    }

    #[test]
    fn test_position_limit_exceeded() {
        let positions = vec![make_position("AAPL", 80, 5000)];
        let rules = vec![make_rule(
            RiskRuleType::PositionLimit,
            100.0,
            RiskAction::Reject,
        )];

        let result =
            RiskControlService::check_order_risk(&positions, &rules, "AAPL", 30, 5000, 1_000_000);
        assert!(!result.allowed);
        assert!(result.reject_reason.unwrap().contains("Position limit"));
    }

    #[test]
    fn test_leverage_limit_allowed() {
        let positions = vec![make_position("AAPL", 100, 5000)];
        let rules = vec![make_rule(
            RiskRuleType::LeverageRatio,
            3.0,
            RiskAction::Reject,
        )];

        // Total position value = 500,000, balance = 500,000 => leverage = 1.0
        let result =
            RiskControlService::check_order_risk(&positions, &rules, "GOOG", 10, 10000, 500_000);
        assert!(result.allowed);
    }

    #[test]
    fn test_leverage_limit_exceeded() {
        let positions = vec![make_position("AAPL", 100, 10000)];
        let rules = vec![make_rule(
            RiskRuleType::LeverageRatio,
            2.0,
            RiskAction::Reject,
        )];

        // Total position value = 1,000,000, balance = 300,000
        // Leverage = 1,000,000 / 300,000 = 3.33 > 2.0
        let result =
            RiskControlService::check_order_risk(&positions, &rules, "GOOG", 0, 0, 300_000);
        assert!(!result.allowed);
    }

    #[test]
    fn test_calculate_margin_ratio() {
        let info = RiskControlService::calculate_margin_ratio(
            1_000_000, // position value
            500_000,   // net assets
            1.5,       // warning threshold
            1.1,       // liquidation threshold
        );

        // margin_ratio = 500k / 1M = 0.5
        assert!((info.margin_ratio - 0.5).abs() < 0.001);
        assert_eq!(info.level, MarginLevel::Liquidation);
    }
}
