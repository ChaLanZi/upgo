use crate::domain::events::AccountEvent;
use crate::domain::user::{User, UserId};
use crate::infrastructure::nats::EventPublisher;
use crate::infrastructure::repositories::user_repository::UserRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

/// Application service for user-related use cases
pub struct UserApplicationService {
    user_repo: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UserApplicationService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            user_repo,
            event_publisher,
        }
    }

    /// Register a new user
    pub async fn register(
        &self,
        email: String,
        password_hash: String,
        nickname: String,
        phone: Option<String>,
    ) -> Result<User> {
        // Check if email already exists
        if self.user_repo.exists_by_email(&email).await? {
            anyhow::bail!("Email already exists");
        }

        // Create user entity
        let user = User::new(email.clone(), password_hash, nickname, phone);

        // Persist
        let saved = self.user_repo.create(&user).await?;

        // Publish event
        self.event_publisher
            .publish(&AccountEvent::UserRegistered {
                user_id: saved.id.0,
                email: saved.email.clone(),
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved)
    }

    /// Get user profile by user ID
    pub async fn get_profile(&self, user_id: &UserId) -> Result<User> {
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: &UserId,
        nickname: Option<String>,
        phone: Option<String>,
    ) -> Result<User> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        user.update_profile(nickname, phone);
        let saved = self.user_repo.update(&user).await?;
        Ok(saved)
    }

    /// Submit KYC verification
    pub async fn submit_kyc(&self, user_id: &UserId) -> Result<User> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let old_status = user.kyc_status.as_str().to_string();
        user.submit_kyc().map_err(|e| anyhow::anyhow!("{}", e))?;

        let saved = self.user_repo.update(&user).await?;

        self.event_publisher
            .publish(&AccountEvent::KycStatusChanged {
                user_id: saved.id.0,
                old_status,
                new_status: saved.kyc_status.as_str().to_string(),
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved)
    }

    /// Approve KYC (admin)
    pub async fn approve_kyc(&self, user_id: &UserId) -> Result<User> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let old_status = user.kyc_status.as_str().to_string();
        user.approve_kyc().map_err(|e| anyhow::anyhow!("{}", e))?;

        let saved = self.user_repo.update(&user).await?;

        self.event_publisher
            .publish(&AccountEvent::KycStatusChanged {
                user_id: saved.id.0,
                old_status,
                new_status: saved.kyc_status.as_str().to_string(),
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved)
    }

    /// Reject KYC (admin)
    pub async fn reject_kyc(&self, user_id: &UserId) -> Result<User> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let old_status = user.kyc_status.as_str().to_string();
        user.reject_kyc().map_err(|e| anyhow::anyhow!("{}", e))?;

        let saved = self.user_repo.update(&user).await?;

        self.event_publisher
            .publish(&AccountEvent::KycStatusChanged {
                user_id: saved.id.0,
                old_status,
                new_status: saved.kyc_status.as_str().to_string(),
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved)
    }
}
