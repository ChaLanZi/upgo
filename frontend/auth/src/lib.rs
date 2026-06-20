//! upgo frontend library — shared UI, Desktop + Web.
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut,)
)]

pub mod api;
pub mod session;
pub mod ui;

pub use api::AuthApiClient;
pub use session::AuthState;
pub use session::SessionManager;

/// Desktop entry point.
#[cfg(feature = "desktop")]
pub fn run_desktop() {
    dioxus::launch(ui::App);
}

/// Web (WASM) entry point.
#[cfg(feature = "web")]
pub fn run_web() {
    dioxus::launch(ui::App);
}
