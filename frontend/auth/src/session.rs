use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use gloo_storage::Storage;

const ACCESS_TOKEN_KEY: &str = "upgo_access_token";
const REFRESH_TOKEN_KEY: &str = "upgo_refresh_token";
const USER_ID_KEY: &str = "upgo_user_id";

/// Current authentication state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// No token stored, user needs to log in
    Unauthenticated,
    /// Token exists but refreshing (loading)
    Loading,
    /// Authenticated with valid tokens
    Authenticated {
        user_id: String,
        access_token: String,
    },
}

/// Manages authentication state and token lifecycle in the browser.
/// Uses gloo-storage for Web, and can be adapted for desktop/mobile via feature flags.
pub struct SessionManager {
    state: Rc<RefCell<AuthState>>,
    on_change: Option<Box<dyn Fn(AuthState)>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let state = if let Some(token) = Self::get_stored(&ACCESS_TOKEN_KEY) {
            AuthState::Authenticated {
                user_id: Self::get_stored(&USER_ID_KEY).unwrap_or_default(),
                access_token: token,
            }
        } else {
            AuthState::Unauthenticated
        };

        Self {
            state: Rc::new(RefCell::new(state)),
            on_change: None,
        }
    }

    /// Initialize the session: try to restore from storage
    pub async fn init(&self) -> AuthState {
        let current = self.state.borrow().clone();
        match &current {
            AuthState::Authenticated { access_token, .. } => {
                // Try to refresh if we have a refresh token
                if let Some(rt) = Self::get_stored(&REFRESH_TOKEN_KEY) {
                    // In production: call refresh API here
                    // For now, check if access token looks valid
                    if access_token.len() > 10 {
                        return current;
                    }
                }
            }
            _ => {}
        }
        self.set_state(AuthState::Unauthenticated);
        AuthState::Unauthenticated
    }

    /// Get current auth state
    pub fn get_state(&self) -> AuthState {
        self.state.borrow().clone()
    }

    /// Set auth state and persist
    pub fn set_authenticated(&self, user_id: &str, access_token: &str, refresh_token: &str) {
        Self::set_stored(&ACCESS_TOKEN_KEY, access_token);
        Self::set_stored(&REFRESH_TOKEN_KEY, refresh_token);
        Self::set_stored(&USER_ID_KEY, user_id);
        self.set_state(AuthState::Authenticated {
            user_id: user_id.to_string(),
            access_token: access_token.to_string(),
        });
    }

    /// Clear auth state and stored tokens
    pub fn logout(&self) {
        Self::remove_stored(&ACCESS_TOKEN_KEY);
        Self::remove_stored(&REFRESH_TOKEN_KEY);
        Self::remove_stored(&USER_ID_KEY);
        self.set_state(AuthState::Unauthenticated);
    }

    /// Handle 401 response: attempt token refresh, return new access token
    pub async fn handle_unauthorized(&self) -> Option<String> {
        let rt = Self::get_stored(&REFRESH_TOKEN_KEY)?;

        // In production: call /api/auth/refresh with refresh_token
        // For now, simulate a refresh
        self.set_state(AuthState::Loading);

        // On failure, logout
        self.logout();
        None
    }

    /// Get stored value (platform-agnostic)
    fn get_stored(key: &str) -> Option<String> {
        #[cfg(target_arch = "wasm32")]
        {
            gloo_storage::LocalStorage::get(key).ok()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Desktop/mobile: use platform-specific storage
            None
        }
    }

    fn set_stored(key: &str, value: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = gloo_storage::LocalStorage::set(key, value);
        }
    }

    fn remove_stored(key: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = gloo_storage::LocalStorage::delete(key);
        }
    }

    fn set_state(&self, new_state: AuthState) {
        *self.state.borrow_mut() = new_state.clone();
        if let Some(ref cb) = self.on_change {
            cb(new_state);
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
