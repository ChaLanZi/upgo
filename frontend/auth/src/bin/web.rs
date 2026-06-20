//! upgo Web — WASM browser application.
//! Built with: wasm-pack build --target web
#![no_main]

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());
    frontend_auth::run_web();
}
