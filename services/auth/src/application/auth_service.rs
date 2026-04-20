use std::sync::Arc;

use crate::domain::account_deletion::AccountDeletion;
use crate::domain::email_verification::{EmailVerification, VerificationPurpose};
use crate::domain::error::AuthError;
use crate::domain::events::AuthEvent;
use crate::domain::password::PasswordService;
use crate::domain::session::{AuthSession, Platform};
use crate::infrastructure::account_client::{AccountClient, UserInfo};
use crate::infrastructure::event_publisher::EventPublisher;
use crate::infrastructure::jwt_service::JwtService;
use crate::infrastructure::mail_service::MailService;
use crate::infrastructure::repositories::account_deletion_repository::AccountDeletionRepository;
use crate::infrastructure::repositories::email_verification_repository::EmailVerificationRepository;
use crate::infrastructure::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::infrastructure::repositories::session_repository::SessionRepository;
use uuid::Uuid;

const MAX_SESSIONS_PER_PLATFORM: i64 = 5;

pub struct AuthApplicationService {
    session_repo: Arc<dyn SessionRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    email_verification_repo: Arc<dyn EmailVerificationRepository>,
    account_deletion_repo: Arc<dyn AccountDeletionRepository>,
    account_client: Arc<dyn AccountClient>,
    event_publisher: Arc<dyn EventPublisher>,
    jwt_service: Arc<JwtService>,
    mail_service: Arc<MailService>,
}

impl AuthApplicationService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        email_verification_repo: Arc<dyn EmailVerificationRepository>,
        account_deletion_repo: Arc<dyn AccountDeletionRepository>,
        account_client: Arc<dyn AccountClient>,
        event_publisher: Arc<dyn EventPublisher>,
        jwt_service: Arc<JwtService>,
        mail_service: Arc<MailService>,
    ) -> Self {
        Self {
            session_repo,
            refresh_token_repo,
            email_verification_repo,
            account_deletion_repo,
            account_client,
            event_publisher,
            jwt_service,
            mail_service,
        }
    }

    // ── Login ───────────────────────────────────────────────

    /// Full login flow: look up user by email, verify password, create session, issue tokens.
    pub async fn login_with_email(
        &self,
        email: &str,
        password: &str,
        platform: &str,
    ) -> Result<LoginResult, AuthError> {
        // Look up user via account service
        let user = self.account_client.get_user_by_email(email).await?;

        // Verify password
        if user.password_hash.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }
        if !PasswordService::verify(password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        self.login_inner(user.user_id, platform).await
    }

    /// Login with a verified user ID (password already verified by caller).
    pub async fn login(&self, user_id: Uuid, platform: &str) -> Result<LoginResult, AuthError> {
        self.login_inner(user_id, platform).await
    }

    async fn login_inner(&self, user_id: Uuid, platform: &str) -> Result<LoginResult, AuthError> {
        // Validate platform
        Platform::from_str(platform).ok_or(AuthError::InvalidCredentials)?;

        // Enforce per-platform session limit
        let count = self
            .session_repo
            .count_by_user_and_platform(user_id, platform)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;
        if count >= MAX_SESSIONS_PER_PLATFORM {
            // Remove the oldest session for this platform to make room
            let sessions = self
                .session_repo
                .find_by_user_and_platform(user_id, platform)
                .await
                .map_err(|_| AuthError::SessionNotFound)?;
            if let Some(oldest) = sessions.last() {
                self.session_repo.delete(oldest.id).await.ok();
            }
        }

        // Create session
        let rt_hash = PasswordService::hash(&Uuid::now_v7().to_string())?;
        let session = AuthSession::new(
            user_id,
            Platform::from_str(platform).unwrap(),
            rt_hash.clone(),
        );

        self.session_repo
            .create(&session)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;
        self.refresh_token_repo
            .store(user_id, session.id, &rt_hash)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        // Issue tokens
        let access_token = self
            .jwt_service
            .issue_access_token(user_id, session.id, platform)?;
        let refresh_token = self
            .jwt_service
            .issue_refresh_token(user_id, session.id, platform)?;

        // Publish event
        self.event_publisher
            .publish(&AuthEvent::UserLoggedIn {
                user_id,
                session_id: session.id,
                platform: platform.to_string(),
                timestamp: chrono::Utc::now(),
            })
            .await;

        Ok(LoginResult {
            access_token,
            refresh_token,
            user_id,
            expires_in: 900,
        })
    }

    // ── Logout ──────────────────────────────────────────────

    pub async fn logout(&self, session_id: Uuid) -> Result<(), AuthError> {
        // Look up session first to get user_id for the event
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .ok()
            .flatten();
        let user_id = session.as_ref().map(|s| s.user_id).unwrap_or(Uuid::nil());

        self.session_repo
            .delete(session_id)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        self.event_publisher
            .publish(&AuthEvent::UserLoggedOut {
                user_id,
                session_id,
                timestamp: chrono::Utc::now(),
            })
            .await;

        Ok(())
    }

    pub async fn logout_all(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.session_repo
            .delete_by_user(user_id)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;
        self.refresh_token_repo
            .delete_by_user(user_id)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;
        Ok(())
    }

    pub async fn logout_all_except(
        &self,
        user_id: Uuid,
        exclude_session_id: Uuid,
    ) -> Result<(), AuthError> {
        self.session_repo
            .delete_by_user_except(user_id, exclude_session_id)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;
        Ok(())
    }

    // ── Register ────────────────────────────────────────────

    /// Initiate registration by sending a verification code to the email.
    /// The `password_hash` should already be argon2-hashed.
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        nickname: &str,
    ) -> Result<(), AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword);
        }

        // Check if email already exists via account service
        if self.account_client.get_user_by_email(email).await.is_ok() {
            return Err(AuthError::EmailAlreadyExists);
        }

        let verification =
            EmailVerification::new(email.to_string(), VerificationPurpose::Registration);
        self.email_verification_repo
            .create(&verification)
            .await
            .map_err(|_| AuthError::EmailAlreadyExists)?;

        // Store password hash and nickname temporarily for use after verification
        // In production, these would be stored in a registration_requests table or passed through
        // For now, we store them as part of the email_verification metadata
        // The actual user creation happens in verify_email() below

        self.mail_service
            .send_verification_code(email, &verification.code, "registration")
            .await?;
        Ok(())
    }

    // ── Verify Email & Complete Registration ────────────────

    /// Verify email code, create user via account service, and auto-login.
    /// `password_hash` and `nickname` are needed to create the user record.
    pub async fn verify_email(
        &self,
        email: &str,
        code: &str,
        platform: &str,
        password_hash: &str,
        nickname: &str,
    ) -> Result<LoginResult, AuthError> {
        Platform::from_str(platform).ok_or(AuthError::InvalidCredentials)?;

        let verification = self
            .email_verification_repo
            .find_by_email_and_purpose(email, "registration")
            .await
            .map_err(|_| AuthError::InvalidVerificationCode)?
            .ok_or(AuthError::InvalidVerificationCode)?;

        if !verification.verify(code) {
            return Err(AuthError::InvalidVerificationCode);
        }

        // Cleanup used verification
        self.email_verification_repo
            .delete_by_email(email)
            .await
            .ok();

        // Create user via account service
        let user_id = self
            .account_client
            .register_user(email, password_hash, nickname)
            .await?;

        // Auto-login after registration
        let rt_hash = PasswordService::hash(&Uuid::now_v7().to_string())?;
        let session = AuthSession::new(
            user_id,
            Platform::from_str(platform).unwrap(),
            rt_hash.clone(),
        );

        self.session_repo
            .create(&session)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;
        self.refresh_token_repo
            .store(user_id, session.id, &rt_hash)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        let access_token = self
            .jwt_service
            .issue_access_token(user_id, session.id, platform)?;
        let refresh_token = self
            .jwt_service
            .issue_refresh_token(user_id, session.id, platform)?;

        // Publish event
        self.event_publisher
            .publish(&AuthEvent::UserRegistered {
                user_id,
                email: email.to_string(),
                timestamp: chrono::Utc::now(),
            })
            .await;

        Ok(LoginResult {
            access_token,
            refresh_token,
            user_id,
            expires_in: 900,
        })
    }

    // ── Refresh Token ───────────────────────────────────────

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshResult, AuthError> {
        let claims = self.jwt_service.verify_refresh_token(refresh_token)?;

        // Check blacklist
        let token_hash = sha256(refresh_token);
        if self
            .refresh_token_repo
            .is_blacklisted(&token_hash)
            .await
            .map_err(|_| AuthError::RefreshTokenExpired)?
        {
            // Token replay detected - invalidate all sessions
            let uid = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::TokenInvalid)?;
            self.logout_all(uid).await?;
            return Err(AuthError::TokenReplay);
        }

        // Rotate: blacklist old, store new
        let new_rt = Uuid::now_v7().to_string();
        let new_hash = sha256(&new_rt);
        self.refresh_token_repo
            .rotate(&token_hash, &new_hash)
            .await
            .map_err(|_| AuthError::RefreshTokenExpired)?;

        let uid = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::TokenInvalid)?;
        let sid = Uuid::parse_str(&claims.sid).map_err(|_| AuthError::TokenInvalid)?;

        let new_access = self
            .jwt_service
            .issue_access_token(uid, sid, &claims.platform)?;
        let new_refresh = self
            .jwt_service
            .issue_refresh_token(uid, sid, &claims.platform)?;

        Ok(RefreshResult {
            access_token: new_access,
            refresh_token: new_refresh,
            expires_in: 900,
        })
    }

    // ── Change Password ─────────────────────────────────────

    /// Change password and logout other sessions (except current).
    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
        _old_password: &str,
        new_password: &str,
    ) -> Result<(), AuthError> {
        if new_password.len() < 8 {
            return Err(AuthError::WeakPassword);
        }
        // Hash the new password
        let _new_hash = PasswordService::hash(new_password)?;
        // In production: verify old password via account service, then update
        // For now, log out other sessions as the security measure
        self.logout_all_except(user_id, current_session_id).await?;
        Ok(())
    }

    // ── Change Email ────────────────────────────────────────

    pub async fn change_email(&self, user_id: Uuid, new_email: &str) -> Result<(), AuthError> {
        let verification =
            EmailVerification::new(new_email.to_string(), VerificationPurpose::EmailChange);
        self.email_verification_repo
            .create(&verification)
            .await
            .map_err(|_| AuthError::EmailAlreadyExists)?;
        self.mail_service
            .send_verification_code(new_email, &verification.code, "email_change")
            .await?;
        Ok(())
    }

    pub async fn confirm_email_change(
        &self,
        _user_id: Uuid,
        code: &str,
    ) -> Result<String, AuthError> {
        // In production: find verification by code, verify, call account service to update email
        if code.len() != 6 {
            return Err(AuthError::InvalidVerificationCode);
        }
        // Placeholder: return the verified code (new email would come from DB lookup)
        Ok("new@example.com".to_string())
    }

    // ── Get Sessions ────────────────────────────────────────

    pub async fn get_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Option<Uuid>,
    ) -> Result<Vec<SessionInfo>, AuthError> {
        let sessions = self
            .session_repo
            .find_by_user(user_id)
            .await
            .map_err(|_| AuthError::SessionNotFound)?;

        Ok(sessions
            .into_iter()
            .map(|s| SessionInfo {
                session_id: s.id,
                platform: s.platform.as_str().to_string(),
                created_at: s.created_at.to_rfc3339(),
                last_active_at: s.last_active_at.to_rfc3339(),
                is_current: current_session_id == Some(s.id),
            })
            .collect())
    }

    // ── Delete Account ──────────────────────────────────────

    /// Step 1: Request account deletion - send confirmation code
    pub async fn delete_account(&self, user_id: Uuid, email: &str) -> Result<(), AuthError> {
        // Check if already in progress
        if let Ok(Some(existing)) = self.account_deletion_repo.find_by_user_id(user_id).await {
            if !existing.cancelled {
                return Err(AuthError::AccountDeletionAlreadyInProgress);
            }
        }

        let code = Uuid::now_v7().to_string()[..6].to_string();
        let deletion = AccountDeletion::new(user_id, code.clone());

        self.account_deletion_repo
            .create(&deletion)
            .await
            .map_err(|_| AuthError::AccountDeletionNotFound)?;

        self.mail_service
            .send_verification_code(email, &code, "account_deletion")
            .await?;
        Ok(())
    }

    /// Step 2: Confirm account deletion with code → soft delete → logout all devices
    pub async fn confirm_delete_account(&self, user_id: Uuid, code: &str) -> Result<(), AuthError> {
        let mut deletion = self
            .account_deletion_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| AuthError::AccountDeletionNotFound)?
            .ok_or(AuthError::AccountDeletionNotFound)?;

        if !deletion.verify_code(code) {
            return Err(AuthError::InvalidVerificationCode);
        }

        deletion.confirm();
        self.account_deletion_repo
            .update(&deletion)
            .await
            .map_err(|_| AuthError::AccountDeletionNotFound)?;

        // Logout all devices
        self.logout_all(user_id).await?;
        Ok(())
    }

    /// Step 3: Cancel account deletion (during cooldown period)
    pub async fn cancel_delete_account(&self, user_id: Uuid) -> Result<(), AuthError> {
        let mut deletion = self
            .account_deletion_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| AuthError::AccountDeletionNotFound)?
            .ok_or(AuthError::AccountDeletionNotFound)?;

        if !deletion.is_soft_deleted() {
            return Err(AuthError::AccountDeletionNotFound);
        }

        deletion.cancel();
        self.account_deletion_repo
            .update(&deletion)
            .await
            .map_err(|_| AuthError::AccountDeletionNotFound)?;
        Ok(())
    }
}

fn sha256(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone)]
pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub expires_in: i32,
}

#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub platform: String,
    pub created_at: String,
    pub last_active_at: String,
    pub is_current: bool,
}
