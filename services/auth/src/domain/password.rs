use crate::domain::error::AuthError;

/// Password hashing service using argon2
pub struct PasswordService;

impl PasswordService {
    /// Hash a password using Argon2id
    pub fn hash(password: &str) -> Result<String, AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword);
        }
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::InvalidPassword)?;
        Ok(hash.to_string())
    }

    /// Verify a password against a stored hash
    pub fn verify(password: &str, hash: &str) -> Result<bool, AuthError> {
        use argon2::{
            password_hash::{PasswordHash, PasswordVerifier},
            Argon2,
        };
        let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError::InvalidPassword)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "testpassword123";
        let hash = PasswordService::hash(password).unwrap();
        assert!(PasswordService::verify(password, &hash).unwrap());
        assert!(!PasswordService::verify("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_weak_password() {
        let result = PasswordService::hash("short");
        assert!(result.is_err());
    }
}
