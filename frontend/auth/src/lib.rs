//! upgo frontend library.
//! Shared types used by both Desktop and Web builds.

pub mod api;
pub mod session;
pub mod ui;

pub use api::AuthApiClient;
pub use session::AuthState;
pub use session::SessionManager;
