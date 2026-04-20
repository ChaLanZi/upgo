use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,         // user_id
    pub sid: String,         // session_id
    pub platform: String,    // desktop | web | mobile
    pub exp: usize,          // expiry timestamp
    pub iat: usize,          // issued at
}

impl JwtClaims {
    pub fn new(user_id: Uuid, session_id: Uuid, platform: &str, expires_in_secs: i64) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.to_string(),
            sid: session_id.to_string(),
            platform: platform.to_string(),
            exp: (now.timestamp() + expires_in_secs) as usize,
            iat: now.timestamp() as usize,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as usize;
        now >= self.exp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = JwtClaims::new(
            Uuid::now_v7(),
            Uuid::now_v7(),
            "desktop",
            900, // 15min
        );
        assert_eq!(claims.platform, "desktop");
        assert!(!claims.is_expired());
    }
}
