//! upgo Desktop — Native desktop application.
//! cargo run -p frontend-auth

fn main() {
    let css = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../web/style.css"));
    let cfg = dioxus::desktop::Config::new().with_custom_head(format!("<style>{css}</style>"));
    dioxus::LaunchBuilder::new()
        .with_cfg(cfg)
        .launch(frontend_auth::ui::App);
}
