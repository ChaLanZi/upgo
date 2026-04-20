#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut,)
)]

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

pub use domain::error::AuthError;
