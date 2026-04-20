// Debug build 时隐藏开发期常见警告，release build 保持严格检查
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut,)
)]

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

/// Re-export common types
pub use domain::error::AccountError;
pub use domain::events::AccountEvent;
