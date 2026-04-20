use crate::domain::error::AuthError;
use crate::domain::jwt::JwtClaims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

/// JWT token service for issuing and validating access and refresh tokens.
pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    /// Issue an access token (short-lived, 15 minutes)
    pub fn issue_access_token(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        platform: &str,
    ) -> Result<String, AuthError> {
        let claims = JwtClaims::new(user_id, session_id, platform, 900); // 15min
        self.sign(&claims)
    }

    /// Issue a refresh token (long-lived, 7 days)
    pub fn issue_refresh_token(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        platform: &str,
    ) -> Result<String, AuthError> {
        let claims = JwtClaims::new(user_id, session_id, platform, 604800); // 7 days
        self.sign(&claims)
    }

    /// Verify and decode an access token
    pub fn verify_access_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let claims = self.decode(token)?;
        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }
        Ok(claims)
    }

    /// Verify and decode a refresh token
    pub fn verify_refresh_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let claims = self.decode(token)?;
        if claims.is_expired() {
            return Err(AuthError::RefreshTokenExpired);
        }
        Ok(claims)
    }

    fn sign(&self, claims: &JwtClaims) -> Result<String, AuthError> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|_| AuthError::TokenInvalid)
    }

    fn decode(&self, token: &str) -> Result<JwtClaims, AuthError> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> JwtService {
        JwtService::new("test-secret-for-testing")
    }

    #[test]
    fn test_issue_and_verify_access_token() {
        let svc = make_service();
        let uid = Uuid::now_v7();
        let sid = Uuid::now_v7();
        let token = svc.issue_access_token(uid, sid, "desktop").unwrap();
        let claims = svc.verify_access_token(&token).unwrap();
        assert_eq!(claims.sub, uid.to_string());
        assert_eq!(claims.sid, sid.to_string());
        assert_eq!(claims.platform, "desktop");
    }

    #[test]
    fn test_issue_and_verify_refresh_token() {
        let svc = make_service();
        let uid = Uuid::now_v7();
        let sid = Uuid::now_v7();
        let token = svc.issue_refresh_token(uid, sid, "web").unwrap();
        let claims = svc.verify_refresh_token(&token).unwrap();
        assert_eq!(claims.platform, "web");
    }

    #[test]
    fn test_invalid_token() {
        let svc = make_service();
        let result = svc.verify_access_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_secrets() {
        let svc1 = JwtService::new("secret1");
        let svc2 = JwtService::new("secret2");
        let token = svc1
            .issue_access_token(Uuid::now_v7(), Uuid::now_v7(), "mobile")
            .unwrap();
        let result = svc2.verify_access_token(&token);
        assert!(result.is_err());
    }
}
