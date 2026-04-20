use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AccountError {
    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Insufficient available balance (frozen funds)")]
    InsufficientAvailableBalance,

    #[error("Insufficient frozen balance")]
    InsufficientFrozenBalance,

    #[error("Invalid amount")]
    InvalidAmount,

    #[error("Currency mismatch")]
    CurrencyMismatch,

    #[error("Insufficient position")]
    InsufficientPosition,

    #[error("Position limit exceeded")]
    PositionLimitExceeded,

    #[error("Industry concentration limit exceeded")]
    IndustryConcentrationExceeded,

    #[error("Leverage limit exceeded")]
    LeverageExceeded,

    #[error("User not found")]
    UserNotFound,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Position not found")]
    PositionNotFound,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid account status transition: {0}")]
    InvalidStatusTransition(String),

    #[error("KYC validation error: {0}")]
    KycError(String),
}
