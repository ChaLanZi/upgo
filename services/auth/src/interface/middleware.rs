//! Axum middleware for JWT authentication.
//!
//! Designed to run on the Pingora Gateway / Axum side.
//! Validates JWT access tokens on protected routes and injects
//! X-User-Id / X-Session-Id headers into downstream requests.
//!
//! White-listed routes (no auth required):
//! - POST /api/auth/login
//! - POST /api/auth/register
//! - POST /api/auth/verify-email
//! - POST /api/auth/refresh

use std::sync::Arc;

use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::domain::error::AuthError;
use crate::infrastructure::jwt_service::JwtService;

/// Routes that do NOT require JWT authentication.
const WHITELIST_PREFIXES: &[&str] = &[
    "/api/auth/login",
    "/api/auth/register",
    "/api/auth/verify-email",
    "/api/auth/refresh",
];

/// JWT authentication middleware state.
#[derive(Clone)]
pub struct AuthMiddleware {
    jwt_service: Arc<JwtService>,
}

impl AuthMiddleware {
    pub fn new(jwt_service: Arc<JwtService>) -> Self {
        Self { jwt_service }
    }

    /// Axum middleware handler.
    ///
    /// Usage in Router:
    /// ```ignore
    /// .route_layer(middleware::from_fn_with_state(state, AuthMiddleware::handle))
    /// ```
    pub async fn handle(
        state: Arc<JwtService>,
        mut req: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let path = req.uri().path();

        // Skip auth for whitelisted routes
        if WHITELIST_PREFIXES
            .iter()
            .any(|&prefix| path.starts_with(prefix))
        {
            return Ok(next.run(req).await);
        }

        // Extract Bearer token
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Verify token
        let claims = state
            .verify_access_token(auth_header)
            .map_err(|e| match e {
                AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                _ => StatusCode::UNAUTHORIZED,
            })?;

        // Inject headers for downstream services
        req.headers_mut().insert(
            "X-User-Id",
            HeaderValue::from_str(&claims.sub).unwrap_or(HeaderValue::from_static("")),
        );
        req.headers_mut().insert(
            "X-Session-Id",
            HeaderValue::from_str(&claims.sid).unwrap_or(HeaderValue::from_static("")),
        );

        Ok(next.run(req).await)
    }
}
