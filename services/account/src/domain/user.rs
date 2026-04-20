use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Value object for User ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// User account status lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    PendingVerification,
    Active,
    Suspended,
    Closed,
}

impl AccountStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountStatus::PendingVerification => "PENDING_VERIFICATION",
            AccountStatus::Active => "ACTIVE",
            AccountStatus::Suspended => "SUSPENDED",
            AccountStatus::Closed => "CLOSED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "PENDING_VERIFICATION" => Some(AccountStatus::PendingVerification),
            "ACTIVE" => Some(AccountStatus::Active),
            "SUSPENDED" => Some(AccountStatus::Suspended),
            "CLOSED" => Some(AccountStatus::Closed),
            _ => None,
        }
    }

    /// Transition to next state. Returns Err if transition is invalid.
    pub fn transition_to(&self, target: AccountStatus) -> Result<AccountStatus, String> {
        match (self, target) {
            (AccountStatus::PendingVerification, AccountStatus::Active) => Ok(target),
            (AccountStatus::Active, AccountStatus::Suspended) => Ok(target),
            (AccountStatus::Suspended, AccountStatus::Active) => Ok(target),
            (AccountStatus::Active, AccountStatus::Closed) => Ok(target),
            (AccountStatus::Suspended, AccountStatus::Closed) => Ok(target),
            _ => Err(format!(
                "Invalid status transition: {:?} -> {:?}",
                self, target
            )),
        }
    }
}

/// KYC verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KycStatus {
    None,
    PendingReview,
    Verified,
    Rejected,
}

impl KycStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            KycStatus::None => "NONE",
            KycStatus::PendingReview => "PENDING_REVIEW",
            KycStatus::Verified => "VERIFIED",
            KycStatus::Rejected => "REJECTED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "NONE" => Some(KycStatus::None),
            "PENDING_REVIEW" => Some(KycStatus::PendingReview),
            "VERIFIED" => Some(KycStatus::Verified),
            "REJECTED" => Some(KycStatus::Rejected),
            _ => None,
        }
    }

    pub fn can_trade(&self) -> bool {
        matches!(self, KycStatus::Verified)
    }
}

/// User aggregate root
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub phone: Option<String>,
    pub nickname: String,
    pub password_hash: String,
    pub kyc_status: KycStatus,
    pub account_status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

impl User {
    pub fn new(
        email: String,
        password_hash: String,
        nickname: String,
        phone: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            email,
            phone,
            nickname,
            password_hash,
            kyc_status: KycStatus::None,
            account_status: AccountStatus::PendingVerification,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    pub fn verify_email(&mut self) -> Result<(), String> {
        self.account_status =
            self.account_status.transition_to(AccountStatus::Active)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn suspend(&mut self) -> Result<(), String> {
        self.account_status =
            self.account_status.transition_to(AccountStatus::Suspended)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.account_status =
            self.account_status.transition_to(AccountStatus::Closed)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn submit_kyc(&mut self) -> Result<(), String> {
        if !matches!(self.kyc_status, KycStatus::None | KycStatus::Rejected) {
            return Err("KYC already submitted or verified".to_string());
        }
        self.kyc_status = KycStatus::PendingReview;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn approve_kyc(&mut self) -> Result<(), String> {
        if !matches!(self.kyc_status, KycStatus::PendingReview) {
            return Err("KYC not in pending review state".to_string());
        }
        self.kyc_status = KycStatus::Verified;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn reject_kyc(&mut self) -> Result<(), String> {
        if !matches!(self.kyc_status, KycStatus::PendingReview) {
            return Err("KYC not in pending review state".to_string());
        }
        self.kyc_status = KycStatus::Rejected;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_profile(&mut self, nickname: Option<String>, phone: Option<String>) {
        if let Some(n) = nickname {
            self.nickname = n;
        }
        if let Some(p) = phone {
            self.phone = Some(p);
        }
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            "TestUser".to_string(),
            None,
        );
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.account_status, AccountStatus::PendingVerification);
        assert_eq!(user.kyc_status, KycStatus::None);
    }

    #[test]
    fn test_email_verification() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            "TestUser".to_string(),
            None,
        );
        assert!(user.verify_email().is_ok());
        assert_eq!(user.account_status, AccountStatus::Active);
    }

    #[test]
    fn test_suspend_active_user() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            "TestUser".to_string(),
            None,
        );
        user.verify_email().unwrap();
        assert!(user.suspend().is_ok());
        assert_eq!(user.account_status, AccountStatus::Suspended);
    }

    #[test]
    fn test_cannot_suspend_pending_user() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            "TestUser".to_string(),
            None,
        );
        assert!(user.suspend().is_err());
    }

    #[test]
    fn test_kyc_lifecycle() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            "TestUser".to_string(),
            None,
        );
        assert!(user.submit_kyc().is_ok());
        assert_eq!(user.kyc_status, KycStatus::PendingReview);
        assert!(user.approve_kyc().is_ok());
        assert_eq!(user.kyc_status, KycStatus::Verified);
        assert!(user.kyc_status.can_trade());
    }

    #[test]
    fn test_kyc_reject_retry() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hash".to_string(),
            "TestUser".to_string(),
            None,
        );
        user.submit_kyc().unwrap();
        user.reject_kyc().unwrap();
        assert_eq!(user.kyc_status, KycStatus::Rejected);
        // Can resubmit after rejection
        assert!(user.submit_kyc().is_ok());
    }
}
