use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

const ACCESS_TOKEN_KEY: &str = "upgo_access_token";
const REFRESH_TOKEN_KEY: &str = "upgo_refresh_token";
const USER_ID_KEY: &str = "upgo_user_id";

/// Current authentication state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    Unauthenticated,
    Loading,
    Authenticated {
        user_id: String,
        access_token: String,
    },
}

/// Cross-platform session manager.
/// - Web: uses browser localStorage via gloo-storage
/// - Desktop: uses filesystem via dirs crate
#[derive(Clone)]
pub struct SessionManager {
    state: Rc<RefCell<AuthState>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let state = if let Some(token) = Self::get_stored(ACCESS_TOKEN_KEY) {
            AuthState::Authenticated {
                user_id: Self::get_stored(USER_ID_KEY).unwrap_or_default(),
                access_token: token,
            }
        } else {
            AuthState::Unauthenticated
        };
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }

    pub fn get_state(&self) -> AuthState {
        self.state.borrow().clone()
    }

    pub fn set_authenticated(&self, user_id: &str, access_token: &str, refresh_token: &str) {
        Self::set_stored(ACCESS_TOKEN_KEY, access_token);
        Self::set_stored(REFRESH_TOKEN_KEY, refresh_token);
        Self::set_stored(USER_ID_KEY, user_id);
        *self.state.borrow_mut() = AuthState::Authenticated {
            user_id: user_id.to_string(),
            access_token: access_token.to_string(),
        };
    }

    pub fn logout(&self) {
        Self::remove_stored(ACCESS_TOKEN_KEY);
        Self::remove_stored(REFRESH_TOKEN_KEY);
        Self::remove_stored(USER_ID_KEY);
        *self.state.borrow_mut() = AuthState::Unauthenticated;
    }

    // ── Platform-specific storage ──────────────────────
    #[cfg(target_arch = "wasm32")]
    fn get_stored(key: &str) -> Option<String> {
        use gloo_storage::Storage;
        gloo_storage::LocalStorage::get(key).ok()
    }
    #[cfg(target_arch = "wasm32")]
    fn set_stored(key: &str, value: &str) {
        use gloo_storage::Storage;
        let _ = gloo_storage::LocalStorage::set(key, value);
    }
    #[cfg(target_arch = "wasm32")]
    fn remove_stored(key: &str) {
        use gloo_storage::Storage;
        let _ = gloo_storage::LocalStorage::delete(key);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_stored(key: &str) -> Option<String> {
        let path = Self::storage_dir().join(key);
        std::fs::read_to_string(path).ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn set_stored(key: &str, value: &str) {
        let path = Self::storage_dir().join(key);
        let _ = std::fs::write(path, value);
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn remove_stored(key: &str) {
        let path = Self::storage_dir().join(key);
        let _ = std::fs::remove_file(path);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn storage_dir() -> std::path::PathBuf {
        let mut dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        dir.push("upgo");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
