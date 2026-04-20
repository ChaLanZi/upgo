use std::sync::Arc;

use contracts::proto::auth::{
    auth_service_server::AuthService, CancelDeleteAccountRequest, CancelDeleteAccountResponse,
    ChangeEmailRequest, ChangeEmailResponse, ChangePasswordRequest, ChangePasswordResponse,
    ConfirmDeleteAccountRequest, ConfirmDeleteAccountResponse, ConfirmEmailChangeRequest,
    ConfirmEmailChangeResponse, DeleteAccountRequest, DeleteAccountResponse, GetSessionsRequest,
    GetSessionsResponse, LoginRequest, LoginResponse, LogoutAllRequest, LogoutAllResponse,
    LogoutRequest, LogoutResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    RegisterResponse, SessionInfo, VerifyEmailRequest, VerifyEmailResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::application::auth_service::AuthApplicationService;
use crate::domain::error::AuthError;
use crate::domain::password::PasswordService;

pub struct AuthGrpcHandler {
    app_service: Arc<AuthApplicationService>,
}

impl AuthGrpcHandler {
    pub fn new(app_service: Arc<AuthApplicationService>) -> Self {
        Self { app_service }
    }

    fn map_auth_error(err: AuthError) -> Status {
        use tonic::Code;
        match err {
            AuthError::InvalidCredentials | AuthError::InvalidPassword => {
                Status::new(Code::Unauthenticated, err.to_string())
            }
            AuthError::WeakPassword => Status::new(Code::InvalidArgument, err.to_string()),
            AuthError::EmailAlreadyExists => Status::new(Code::AlreadyExists, err.to_string()),
            AuthError::AccountSuspended | AuthError::AccountDeleted => {
                Status::new(Code::PermissionDenied, err.to_string())
            }
            AuthError::AccountNotFound | AuthError::UserNotFound => {
                Status::new(Code::NotFound, err.to_string())
            }
            AuthError::SessionNotFound => Status::new(Code::NotFound, err.to_string()),
            AuthError::TokenExpired | AuthError::RefreshTokenExpired => {
                Status::new(Code::Unauthenticated, err.to_string())
            }
            AuthError::TokenInvalid => Status::new(Code::Unauthenticated, err.to_string()),
            AuthError::TokenReplay => Status::new(Code::Unauthenticated, err.to_string()),
            AuthError::InvalidVerificationCode | AuthError::VerificationCodeExpired => {
                Status::new(Code::InvalidArgument, err.to_string())
            }
            AuthError::AccountDeletionNotFound => Status::new(Code::NotFound, err.to_string()),
            AuthError::AccountDeletionAlreadyInProgress => {
                Status::new(Code::FailedPrecondition, err.to_string())
            }
        }
    }
}

#[tonic::async_trait]
impl AuthService for AuthGrpcHandler {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        // Validate platform
        if !["desktop", "web", "mobile"].contains(&req.platform.as_str()) {
            return Err(Status::new(
                tonic::Code::InvalidArgument,
                "Invalid platform",
            ));
        }

        // Full login flow: lookup user + verify password + create session + issue tokens
        let result = self
            .app_service
            .login_with_email(&req.email, &req.password, &req.platform)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(LoginResponse {
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            user_id: result.user_id.to_string(),
            email: req.email,
            nickname: String::new(),
            expires_in: result.expires_in,
        }))
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        self.app_service
            .register(&req.email, &req.password, &req.nickname)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(RegisterResponse {
            message: "Verification code sent to your email".to_string(),
        }))
    }

    async fn verify_email(
        &self,
        request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailResponse>, Status> {
        let req = request.into_inner();

        // In production: get password_hash and nickname from temporary storage
        // For dev: use the email-derived password_hash and extract nickname from email
        let password_hash =
            crate::domain::password::PasswordService::hash(&req.email).unwrap_or_default();
        let nickname = req.email.split('@').next().unwrap_or("user").to_string();

        // Verify email code, create user via account service, and auto-login
        let result = self
            .app_service
            .verify_email(
                &req.email,
                &req.code,
                &req.platform,
                &password_hash,
                &nickname,
            )
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(VerifyEmailResponse {
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            user_id: result.user_id.to_string(),
            email: req.email,
            nickname,
        }))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|_| Status::invalid_argument("Invalid session_id"))?;

        self.app_service
            .logout(session_id)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(LogoutResponse { success: true }))
    }

    async fn logout_all(
        &self,
        request: Request<LogoutAllRequest>,
    ) -> Result<Response<LogoutAllResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        self.app_service
            .logout_all(user_id)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(LogoutAllResponse { success: true }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();

        let result = self
            .app_service
            .refresh_token(&req.refresh_token)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(RefreshTokenResponse {
            access_token: result.access_token,
            refresh_token: result.refresh_token,
            expires_in: result.expires_in,
        }))
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        // Extract current session_id from gRPC metadata (injected by gateway middleware)
        // NOTE: must extract metadata before calling into_inner() which consumes request
        let current_session_id = request
            .metadata()
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok())
            .unwrap_or_else(Uuid::nil);

        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        self.app_service
            .change_password(
                user_id,
                current_session_id,
                &req.old_password,
                &req.new_password,
            )
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(ChangePasswordResponse { success: true }))
    }

    async fn change_email(
        &self,
        request: Request<ChangeEmailRequest>,
    ) -> Result<Response<ChangeEmailResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        self.app_service
            .change_email(user_id, &req.new_email)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(ChangeEmailResponse {
            message: "Verification code sent to new email".to_string(),
        }))
    }

    async fn confirm_email_change(
        &self,
        request: Request<ConfirmEmailChangeRequest>,
    ) -> Result<Response<ConfirmEmailChangeResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let new_email = self
            .app_service
            .confirm_email_change(user_id, &req.code)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(ConfirmEmailChangeResponse {
            success: true,
            new_email,
        }))
    }

    async fn delete_account(
        &self,
        request: Request<DeleteAccountRequest>,
    ) -> Result<Response<DeleteAccountResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        // In production: get email from account service
        // For now, use a placeholder email
        let email = format!("user_{}@upgo.local", user_id);

        self.app_service
            .delete_account(user_id, &email)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(DeleteAccountResponse {
            message: "Deletion confirmation code sent to your email".to_string(),
        }))
    }

    async fn confirm_delete_account(
        &self,
        request: Request<ConfirmDeleteAccountRequest>,
    ) -> Result<Response<ConfirmDeleteAccountResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        self.app_service
            .confirm_delete_account(user_id, &req.code)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(ConfirmDeleteAccountResponse {
            success: true,
            deleted_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    async fn cancel_delete_account(
        &self,
        request: Request<CancelDeleteAccountRequest>,
    ) -> Result<Response<CancelDeleteAccountResponse>, Status> {
        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        self.app_service
            .cancel_delete_account(user_id)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(CancelDeleteAccountResponse { success: true }))
    }

    async fn get_sessions(
        &self,
        request: Request<GetSessionsRequest>,
    ) -> Result<Response<GetSessionsResponse>, Status> {
        // Extract current session_id from metadata (before consuming request)
        let current_session_id = request
            .metadata()
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok());

        let req = request.into_inner();
        let user_id = Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let sessions = self
            .app_service
            .get_sessions(user_id, current_session_id)
            .await
            .map_err(Self::map_auth_error)?;

        Ok(Response::new(GetSessionsResponse {
            sessions: sessions
                .into_iter()
                .map(|s| SessionInfo {
                    session_id: s.session_id.to_string(),
                    platform: s.platform,
                    created_at: s.created_at,
                    last_active_at: s.last_active_at,
                    is_current: s.is_current,
                })
                .collect(),
        }))
    }
}
