use crate::domain::error::AccountError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Currency type (CNY, USD, HKD)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    CNY,
    USD,
    HKD,
}

impl Currency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Currency::CNY => "CNY",
            Currency::USD => "USD",
            Currency::HKD => "HKD",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CNY" => Some(Currency::CNY),
            "USD" => Some(Currency::USD),
            "HKD" => Some(Currency::HKD),
            _ => None,
        }
    }
}

/// Money value object - stored in cents (fen) to avoid float precision issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    pub amount: i64,  // in smallest unit (cents/fen)
    pub currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    pub fn zero(currency: Currency) -> Self {
        Self { amount: 0, currency }
    }

    pub fn add(&self, other: &Money) -> Result<Money, AccountError> {
        if self.currency != other.currency {
            return Err(AccountError::CurrencyMismatch);
        }
        Ok(Money::new(self.amount + other.amount, self.currency))
    }

    pub fn subtract(&self, other: &Money) -> Result<Money, AccountError> {
        if self.currency != other.currency {
            return Err(AccountError::CurrencyMismatch);
        }
        if self.amount < other.amount {
            return Err(AccountError::InsufficientBalance);
        }
        Ok(Money::new(self.amount - other.amount, self.currency))
    }

    pub fn negate(&self) -> Money {
        Money::new(-self.amount, self.currency)
    }
}

/// Transaction type for fund operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    FundFrozen,
    FundUnfrozen,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Deposit => "DEPOSIT",
            TransactionType::Withdrawal => "WITHDRAWAL",
            TransactionType::FundFrozen => "FUND_FROZEN",
            TransactionType::FundUnfrozen => "FUND_UNFROZEN",
        }
    }
}

/// Fund transaction record
#[derive(Debug, Clone)]
pub struct FundTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub balance_before: i64,
    pub balance_after: i64,
    pub order_id: Option<String>,
    pub remark: String,
    pub created_at: DateTime<Utc>,
}

/// Fund account aggregate root
#[derive(Debug, Clone)]
pub struct FundAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub currency: Currency,
    pub balance: i64,          // total balance in cents
    pub frozen_balance: i64,   // frozen amount in cents
    pub version: i32,
}

impl FundAccount {
    pub fn new(user_id: Uuid, currency: Currency) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id,
            currency,
            balance: 0,
            frozen_balance: 0,
            version: 1,
        }
    }

    pub fn available_balance(&self) -> i64 {
        self.balance - self.frozen_balance
    }

    /// Deposit money into account
    pub fn deposit(&mut self, amount: i64) -> Result<i64, AccountError> {
        if amount <= 0 {
            return Err(AccountError::InvalidAmount);
        }
        self.balance += amount;
        Ok(self.balance)
    }

    /// Withdraw money from account
    pub fn withdraw(&mut self, amount: i64) -> Result<i64, AccountError> {
        if amount <= 0 {
            return Err(AccountError::InvalidAmount);
        }
        if self.available_balance() < amount {
            return Err(AccountError::InsufficientBalance);
        }
        self.balance -= amount;
        Ok(self.balance)
    }

    /// Freeze funds for an order
    pub fn freeze(&mut self, amount: i64) -> Result<i64, AccountError> {
        if amount <= 0 {
            return Err(AccountError::InvalidAmount);
        }
        if self.available_balance() < amount {
            return Err(AccountError::InsufficientAvailableBalance);
        }
        self.frozen_balance += amount;
        Ok(self.frozen_balance)
    }

    /// Unfreeze funds when order is cancelled
    pub fn unfreeze(&mut self, amount: i64) -> Result<i64, AccountError> {
        if amount <= 0 {
            return Err(AccountError::InvalidAmount);
        }
        if self.frozen_balance < amount {
            return Err(AccountError::InsufficientFrozenBalance);
        }
        self.frozen_balance -= amount;
        Ok(self.frozen_balance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_account() -> FundAccount {
        FundAccount::new(Uuid::now_v7(), Currency::CNY)
    }

    #[test]
    fn test_deposit() {
        let mut acc = create_account();
        assert!(acc.deposit(10000).is_ok());
        assert_eq!(acc.balance, 10000);
        assert_eq!(acc.available_balance(), 10000);
    }

    #[test]
    fn test_withdraw_sufficient() {
        let mut acc = create_account();
        acc.deposit(10000).unwrap();
        assert!(acc.withdraw(5000).is_ok());
        assert_eq!(acc.balance, 5000);
    }

    #[test]
    fn test_withdraw_insufficient() {
        let mut acc = create_account();
        acc.deposit(3000).unwrap();
        assert!(acc.withdraw(5000).is_err());
        assert_eq!(acc.balance, 3000);
    }

    #[test]
    fn test_freeze_and_unfreeze() {
        let mut acc = create_account();
        acc.deposit(10000).unwrap();
        assert!(acc.freeze(3000).is_ok());
        assert_eq!(acc.frozen_balance, 3000);
        assert_eq!(acc.available_balance(), 7000);

        assert!(acc.unfreeze(3000).is_ok());
        assert_eq!(acc.frozen_balance, 0);
        assert_eq!(acc.available_balance(), 10000);
    }

    #[test]
    fn test_freeze_exceeds_available() {
        let mut acc = create_account();
        acc.deposit(5000).unwrap();
        acc.freeze(3000).unwrap();
        // Only 2000 available
        assert!(matches!(
            acc.freeze(3000),
            Err(AccountError::InsufficientAvailableBalance)
        ));
    }

    #[test]
    fn test_invalid_amount() {
        let mut acc = create_account();
        assert!(matches!(
            acc.deposit(0),
            Err(AccountError::InvalidAmount)
        ));
        assert!(matches!(
            acc.withdraw(-100),
            Err(AccountError::InvalidAmount)
        ));
    }

    #[test]
    fn test_optimistic_lock_version() {
        let mut acc = create_account();
        assert_eq!(acc.version, 1);
        acc.deposit(10000).unwrap();
        acc.version += 1; // Simulates increment on update
        assert_eq!(acc.version, 2);
    }
}
