use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Account deletion tracking with cooldown period
#[derive(Debug, Clone)]
pub struct AccountDeletion {
    pub user_id: Uuid,
    pub confirmation_code: String,
    pub requested_at: DateTime<Utc>,
    pub soft_deleted_at: Option<DateTime<Utc>>,
    pub permanent_delete_at: Option<DateTime<Utc>>,
    pub cancelled: bool,
}

impl Default for AccountDeletion {
    fn default() -> Self {
        Self {
            user_id: Uuid::nil(),
            confirmation_code: String::new(),
            requested_at: Utc::now(),
            soft_deleted_at: None,
            permanent_delete_at: None,
            cancelled: false,
        }
    }
}

impl AccountDeletion {
    const COOLDOWN_DAYS: i64 = 30;

    pub fn new(user_id: Uuid, confirmation_code: String) -> Self {
        Self {
            user_id,
            confirmation_code,
            requested_at: Utc::now(),
            soft_deleted_at: None,
            permanent_delete_at: None,
            cancelled: false,
        }
    }

    pub fn confirm(&mut self) {
        let now = Utc::now();
        self.soft_deleted_at = Some(now);
        self.permanent_delete_at = Some(now + chrono::Duration::days(Self::COOLDOWN_DAYS));
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.soft_deleted_at = None;
        self.permanent_delete_at = None;
    }

    pub fn is_soft_deleted(&self) -> bool {
        self.soft_deleted_at.is_some() && !self.cancelled
    }

    pub fn is_ready_for_permanent_delete(&self) -> bool {
        if let Some(permanent_at) = self.permanent_delete_at {
            Utc::now() >= permanent_at && !self.cancelled
        } else {
            false
        }
    }

    pub fn verify_code(&self, code: &str) -> bool {
        self.confirmation_code == code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deletion_lifecycle() {
        let mut d = AccountDeletion::new(Uuid::now_v7(), "123456".into());
        assert!(!d.is_soft_deleted());

        d.confirm();
        assert!(d.is_soft_deleted());
        assert!(!d.is_ready_for_permanent_delete()); // 30 days not passed

        d.cancel();
        assert!(!d.is_soft_deleted());
    }

    #[test]
    fn test_verify_code() {
        let d = AccountDeletion::new(Uuid::now_v7(), "654321".into());
        assert!(d.verify_code("654321"));
        assert!(!d.verify_code("000000"));
    }
}
