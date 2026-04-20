use chrono::{DateTime, Utc};
use rand::Rng;

/// Email verification code
#[derive(Debug, Clone)]
pub struct EmailVerification {
    pub email: String,
    pub code: String,
    pub purpose: VerificationPurpose,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationPurpose {
    Registration,
    EmailChange,
    AccountDeletion,
}

impl EmailVerification {
    pub fn new(email: String, purpose: VerificationPurpose) -> Self {
        let code = Self::generate_code();
        let now = Utc::now();
        Self {
            email,
            code,
            purpose,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(10),
        }
    }

    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn verify(&self, code: &str) -> bool {
        !self.is_expired() && self.code == code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_generation() {
        let v =
            EmailVerification::new("test@example.com".into(), VerificationPurpose::Registration);
        assert_eq!(v.code.len(), 6);
        assert!(v.code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_code_verify() {
        let v =
            EmailVerification::new("test@example.com".into(), VerificationPurpose::Registration);
        assert!(v.verify(&v.code));
        assert!(!v.verify("000000"));
    }
}
