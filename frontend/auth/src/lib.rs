// Debug build 时隐藏开发期常见警告
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut,)
)]

pub mod session;
pub mod api;
pub mod ui;

/// Re-export auth state types
pub use session::AuthState;
pub use session::SessionManager;
pub use api::AuthApiClient;
