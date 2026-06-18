//! upgo frontend — Dioxus Desktop application.
//! Native desktop app, no browser or WASM required.

pub mod api;
pub mod session;
pub mod ui;

pub use api::AuthApiClient;
pub use session::AuthState;
pub use session::SessionManager;

/// Desktop entry point.
fn main() {
    dioxus::launch(ui::App);
}
