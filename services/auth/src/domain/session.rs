use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Web,
    Mobile,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Desktop => "desktop",
            Platform::Web => "web",
            Platform::Mobile => "mobile",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "desktop" => Some(Platform::Desktop),
            "web" => Some(Platform::Web),
            "mobile" => Some(Platform::Mobile),
            _ => None,
        }
    }
}

/// Auth session aggregate root
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: Platform,
    pub refresh_token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

impl AuthSession {
    pub fn new(user_id: Uuid, platform: Platform, refresh_token_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            user_id,
            platform,
            refresh_token_hash,
            created_at: now,
            expires_at: now + chrono::Duration::days(7),
            last_active_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = AuthSession::new(Uuid::now_v7(), Platform::Desktop, "hash".to_string());
        assert_eq!(session.platform, Platform::Desktop);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_platform_roundtrip() {
        for p in &[Platform::Desktop, Platform::Web, Platform::Mobile] {
            assert_eq!(Platform::from_str(p.as_str()), Some(p.clone()));
        }
    }
}
