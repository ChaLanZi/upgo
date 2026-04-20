use crate::domain::error::AccountError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Position status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionStatus {
    Open,
    Closed,
}

impl std::fmt::Display for PositionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionStatus::Open => write!(f, "OPEN"),
            PositionStatus::Closed => write!(f, "CLOSED"),
        }
    }
}

impl PositionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PositionStatus::Open => "OPEN",
            PositionStatus::Closed => "CLOSED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "OPEN" => Some(PositionStatus::Open),
            "CLOSED" => Some(PositionStatus::Closed),
            _ => None,
        }
    }
}

/// Position aggregate root
#[derive(Debug, Clone)]
pub struct Position {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub quantity: i64,      // number of shares
    pub cost_price: i64,    // weighted average cost in cents (fen)
    pub current_price: i64, // latest price in cents
    pub status: PositionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl Position {
    pub fn new(user_id: Uuid, symbol: &str, quantity: i64, price: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            user_id,
            symbol: symbol.to_string(),
            quantity,
            cost_price: price,
            current_price: price,
            status: PositionStatus::Open,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// Market value in cents
    pub fn market_value(&self) -> i64 {
        self.quantity * self.current_price
    }

    /// Unrealized P&L in cents
    pub fn unrealized_pnl(&self) -> i64 {
        (self.current_price - self.cost_price) * self.quantity
    }

    /// Add to existing position (buy more), update weighted average cost
    pub fn add(&mut self, quantity: i64, price: i64) {
        let total_cost = (self.quantity * self.cost_price) + (quantity * price);
        self.quantity += quantity;
        self.cost_price = total_cost / self.quantity;
        self.current_price = price;
        self.updated_at = Utc::now();
    }

    /// Reduce position (sell), returns realized P&L in cents
    pub fn reduce(&mut self, quantity: i64, price: i64) -> Result<i64, AccountError> {
        if quantity > self.quantity {
            return Err(AccountError::InsufficientPosition);
        }

        let realized_pnl = (price - self.cost_price) * quantity;
        self.quantity -= quantity;
        self.current_price = price;
        self.updated_at = Utc::now();

        if self.quantity == 0 {
            self.status = PositionStatus::Closed;
        }

        Ok(realized_pnl)
    }

    /// Update current price from market data
    pub fn update_price(&mut self, price: i64) {
        self.current_price = price;
        self.updated_at = Utc::now();
    }
}

/// Position change history entry
#[derive(Debug, Clone)]
pub struct PositionHistory {
    pub id: Uuid,
    pub position_id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub change_type: String, // BUY | SELL | ADJUST
    pub change_quantity: i64,
    pub quantity_after: i64,
    pub price: i64,
    pub created_at: DateTime<Utc>,
}

impl PositionHistory {
    pub fn new(
        position_id: Uuid,
        user_id: Uuid,
        symbol: &str,
        change_type: &str,
        change_quantity: i64,
        quantity_after: i64,
        price: i64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            position_id,
            user_id,
            symbol: symbol.to_string(),
            change_type: change_type.to_string(),
            change_quantity,
            quantity_after,
            price,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_open_position() {
        let pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        assert_eq!(pos.quantity, 100);
        assert_eq!(pos.cost_price, 5000);
        assert_eq!(pos.status, PositionStatus::Open);
    }

    #[test]
    fn test_market_value_and_pnl() {
        let mut pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        pos.update_price(5500);
        assert_eq!(pos.market_value(), 550_000);
        assert_eq!(pos.unrealized_pnl(), 50_000); // (5500-5000)*100
    }

    #[test]
    fn test_weighted_average_cost() {
        let mut pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        // Buy another 50 shares at 6000
        pos.add(50, 6000);
        assert_eq!(pos.quantity, 150);
        // Weighted average: (100*5000 + 50*6000) / 150 = 5333
        assert_eq!(pos.cost_price, 5333);
    }

    #[test]
    fn test_reduce_position() {
        let mut pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        let result = pos.reduce(30, 5500).unwrap();
        assert_eq!(pos.quantity, 70);
        // Realized P&L: (5500-5000)*30 = 15000
        assert_eq!(result, 15_000);
    }

    #[test]
    fn test_close_position() {
        let mut pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        pos.reduce(100, 5500).unwrap();
        assert_eq!(pos.quantity, 0);
        assert_eq!(pos.status, PositionStatus::Closed);
    }

    #[test]
    fn test_reduce_excessive() {
        let mut pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        assert!(matches!(
            pos.reduce(150, 5500),
            Err(AccountError::InsufficientPosition)
        ));
    }

    #[test]
    fn test_price_update() {
        let mut pos = Position::new(Uuid::now_v7(), "AAPL", 100, 5000);
        pos.update_price(5200);
        assert_eq!(pos.current_price, 5200);
        assert_eq!(pos.unrealized_pnl(), 20_000); // (5200-5000)*100
    }
}
