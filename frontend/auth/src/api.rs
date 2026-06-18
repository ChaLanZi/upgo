use serde::{Deserialize, Serialize};
use gloo_storage::Storage;

/// Auth API client for communicating with the auth service.
/// Platform-agnostic: uses reqwest on native and gloo-net on WASM.
pub struct AuthApiClient {
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub nickname: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub code: String,
    pub platform: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub expires_in: i32,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl AuthApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// POST /api/auth/login
    pub async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, String> {
        let url = format!("{}/api/auth/login", self.base_url);
        let resp = reqwest::Client::new()
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status().is_success() {
            resp.json::<AuthResponse>()
                .await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            let err = resp.json::<ErrorResponse>().await.ok();
            Err(err.map(|e| e.error).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// POST /api/auth/register
    pub async fn register(&self, req: &RegisterRequest) -> Result<(), String> {
        let url = format!("{}/api/auth/register", self.base_url);
        let resp = reqwest::Client::new()
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let err = resp.json::<ErrorResponse>().await.ok();
            Err(err.map(|e| e.error).unwrap_or_else(|| "Registration failed".to_string()))
        }
    }

    /// POST /api/auth/verify-email
    pub async fn verify_email(&self, req: &VerifyEmailRequest) -> Result<AuthResponse, String> {
        let url = format!("{}/api/auth/verify-email", self.base_url);
        let resp = reqwest::Client::new()
            .post(&url)
            .json(req)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status().is_success() {
            resp.json::<AuthResponse>()
                .await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            let err = resp.json::<ErrorResponse>().await.ok();
            Err(err.map(|e| e.error).unwrap_or_else(|| "Verification failed".to_string()))
        }
    }

    /// POST /api/auth/refresh
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthResponse, String> {
        let url = format!("{}/api/auth/refresh", self.base_url);
        #[derive(Serialize)]
        struct RefreshBody {
            refresh_token: String,
        }

        let resp = reqwest::Client::new()
            .post(&url)
            .json(&RefreshBody {
                refresh_token: refresh_token.to_string(),
            })
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if resp.status().is_success() {
            resp.json::<AuthResponse>()
                .await
                .map_err(|e| format!("Parse error: {}", e))
        } else {
            Err("Token refresh failed".to_string())
        }
    }

    /// POST /api/auth/logout
    pub async fn logout(&self, session_id: &str) -> Result<(), String> {
        let url = format!("{}/api/auth/logout", self.base_url);
        #[derive(Serialize)]
        struct LogoutBody {
            session_id: String,
        }

        let _ = reqwest::Client::new()
            .post(&url)
            .json(&LogoutBody {
                session_id: session_id.to_string(),
            })
            .send()
            .await;

        Ok(())
    }
}
