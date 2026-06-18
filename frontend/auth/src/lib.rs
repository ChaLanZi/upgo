//! upgo frontend — Dioxus WASM application.
//! Debug build hides common dev warnings; release build keeps strict checks.
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

/// WASM entry point — called by index.html.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::launch(ui::App);
}
