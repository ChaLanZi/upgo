use crate::application::UserApplicationService;
use crate::domain::user::UserId;
use std::sync::Arc;

/// gRPC handler for UserService
pub struct UserGrpcHandler {
    user_service: Arc<UserApplicationService>,
}

impl UserGrpcHandler {
    pub fn new(user_service: Arc<UserApplicationService>) -> Self {
        Self { user_service }
    }

    pub async fn handle_register(
        &self,
        email: String,
        password_hash: String,
        nickname: String,
        phone: Option<String>,
    ) -> Result<String, String> {
        self.user_service
            .register(email, password_hash, nickname, phone)
            .await
            .map(|u| u.id.to_string())
            .map_err(|e| e.to_string())
    }

    pub async fn handle_get_profile(
        &self,
        user_id: &UserId,
    ) -> Result<crate::domain::user::User, String> {
        self.user_service
            .get_profile(user_id)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn handle_submit_kyc(&self, user_id: &UserId) -> Result<String, String> {
        self.user_service
            .submit_kyc(user_id)
            .await
            .map(|u| u.kyc_status.as_str().to_string())
            .map_err(|e| e.to_string())
    }

    pub async fn handle_approve_kyc(&self, user_id: &UserId) -> Result<String, String> {
        self.user_service
            .approve_kyc(user_id)
            .await
            .map(|u| u.kyc_status.as_str().to_string())
            .map_err(|e| e.to_string())
    }

    pub async fn handle_reject_kyc(&self, user_id: &UserId) -> Result<String, String> {
        self.user_service
            .reject_kyc(user_id)
            .await
            .map(|u| u.kyc_status.as_str().to_string())
            .map_err(|e| e.to_string())
    }
}
