use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::error::AuthError;

/// gRPC client for interacting with the Account service.
///
/// In production, this uses the generated tonic client from `contracts::proto::user`.
/// For development, a placeholder implementation is provided.
#[async_trait]
pub trait AccountClient: Send + Sync {
    /// Register a new user with the account service.
    /// `password_hash` should already be argon2-hashed by the auth service.
    async fn register_user(
        &self,
        email: &str,
        password_hash: &str,
        nickname: &str,
    ) -> Result<Uuid, AuthError>;

    /// Look up a user by email, returning (user_id, password_hash, nickname).
    async fn get_user_by_email(&self, email: &str) -> Result<UserInfo, AuthError>;

    /// Look up a user by ID.
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<UserInfo, AuthError>;

    /// Update user's email (after verification).
    async fn update_email(&self, user_id: Uuid, new_email: &str) -> Result<(), AuthError>;

    /// Update user's password hash.
    async fn update_password_hash(
        &self,
        user_id: Uuid,
        new_password_hash: &str,
    ) -> Result<(), AuthError>;

    /// Soft-delete a user (set deleted_at).
    async fn soft_delete_user(&self, user_id: Uuid, deleted_at: &str) -> Result<(), AuthError>;
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub email: String,
    pub nickname: String,
    pub password_hash: String,
}

/// Tonic-based gRPC client that calls the account service's UserService.
pub struct GrpcAccountClient {
    /// In production: `contracts::proto::user::UserServiceClient<tonic::transport::Channel>`
    /// For now, this is a placeholder that stores the connection address.
    _addr: String,
}

impl GrpcAccountClient {
    pub fn new(addr: &str) -> Self {
        Self {
            _addr: addr.to_string(),
        }
    }
}

#[async_trait]
impl AccountClient for GrpcAccountClient {
    async fn register_user(
        &self,
        _email: &str,
        _password_hash: &str,
        _nickname: &str,
    ) -> Result<Uuid, AuthError> {
        // TODO: Implement gRPC call when account service is running:
        // let mut client = UserServiceClient::new(/* channel */);
        // let resp = client.register(RegisterRequest {
        //     email: email.to_string(),
        //     password: password_hash.to_string(),
        //     nickname: nickname.to_string(),
        //     phone: None,
        // }).await?;
        // Ok(Uuid::parse_str(&resp.into_inner().user_id)?)
        Err(AuthError::AccountNotFound)
    }

    async fn get_user_by_email(&self, _email: &str) -> Result<UserInfo, AuthError> {
        // TODO: Implement gRPC call
        Err(AuthError::UserNotFound)
    }

    async fn get_user_by_id(&self, _user_id: Uuid) -> Result<UserInfo, AuthError> {
        Err(AuthError::UserNotFound)
    }

    async fn update_email(&self, _user_id: Uuid, _new_email: &str) -> Result<(), AuthError> {
        Ok(())
    }

    async fn update_password_hash(
        &self,
        _user_id: Uuid,
        _new_password_hash: &str,
    ) -> Result<(), AuthError> {
        Ok(())
    }

    async fn soft_delete_user(&self, _user_id: Uuid, _deleted_at: &str) -> Result<(), AuthError> {
        Ok(())
    }
}

/// Placeholder account client for development — uses deterministic UUIDs from email.
/// This allows basic end-to-end testing without needing a running account service.
pub struct DevAccountClient;

#[async_trait]
impl AccountClient for DevAccountClient {
    async fn register_user(
        &self,
        _email: &str,
        _password_hash: &str,
        _nickname: &str,
    ) -> Result<Uuid, AuthError> {
        // Generate a deterministic UUID from the email
        Ok(email_to_uuid(_email))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<UserInfo, AuthError> {
        Ok(UserInfo {
            user_id: email_to_uuid(email),
            email: email.to_string(),
            nickname: email.split('@').next().unwrap_or("user").to_string(),
            password_hash: String::new(),
        })
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<UserInfo, AuthError> {
        Ok(UserInfo {
            user_id,
            email: format!("user_{}@upgo.local", user_id),
            nickname: "User".to_string(),
            password_hash: String::new(),
        })
    }

    async fn update_email(&self, _user_id: Uuid, _new_email: &str) -> Result<(), AuthError> {
        Ok(())
    }

    async fn update_password_hash(
        &self,
        _user_id: Uuid,
        _new_password_hash: &str,
    ) -> Result<(), AuthError> {
        Ok(())
    }

    async fn soft_delete_user(&self, _user_id: Uuid, _deleted_at: &str) -> Result<(), AuthError> {
        Ok(())
    }
}

fn email_to_uuid(email: &str) -> Uuid {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    email.hash(&mut hasher);
    let bytes = hasher.finish().to_le_bytes();
    Uuid::from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
        bytes[6], bytes[7],
    ])
}
