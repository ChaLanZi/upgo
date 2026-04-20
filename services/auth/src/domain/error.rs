use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Weak password (min 8 characters)")]
    WeakPassword,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Account suspended")]
    AccountSuspended,

    #[error("Account deleted")]
    AccountDeleted,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid")]
    TokenInvalid,

    #[error("Token replay detected")]
    TokenReplay,

    #[error("Refresh token expired")]
    RefreshTokenExpired,

    #[error("Invalid verification code")]
    InvalidVerificationCode,

    #[error("Verification code expired")]
    VerificationCodeExpired,

    #[error("User not found")]
    UserNotFound,

    #[error("Account deletion request not found")]
    AccountDeletionNotFound,

    #[error("Account deletion already in progress")]
    AccountDeletionAlreadyInProgress,
}
