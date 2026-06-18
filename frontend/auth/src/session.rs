use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::PathBuf;
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

/// Manages authentication state and token lifecycle.
/// Uses local file-based storage for Desktop.
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

    // ── File-based storage helpers ─────────────────────
    fn storage_dir() -> PathBuf {
        let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push("upgo");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    fn file_path(key: &str) -> PathBuf {
        let mut path = Self::storage_dir();
        path.push(key);
        path
    }

    fn get_stored(key: &str) -> Option<String> {
        let path = Self::file_path(key);
        std::fs::read_to_string(path).ok()
    }

    fn set_stored(key: &str, value: &str) {
        let path = Self::file_path(key);
        let _ = std::fs::write(path, value);
    }

    fn remove_stored(key: &str) {
        let path = Self::file_path(key);
        let _ = std::fs::remove_file(path);
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
