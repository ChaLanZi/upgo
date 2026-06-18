//! Static file serving with embedded frontend assets.
//! Frontend files are compiled into the binary via rust-embed.

use axum::body::Body;
use axum::response::Response;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../frontend/web"]
struct Assets;

/// Serve a static file or SPA fallback (index.html).
pub fn serve(path: &str) -> Response<Body> {
    let asset_path = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    match Assets::get(asset_path) {
        Some(data) => {
            let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
            Response::builder()
                .header("Content-Type", mime.as_ref())
                .header("Cache-Control", "public, max-age=3600")
                .body(Body::from(data.data))
                .unwrap()
        }
        None => {
            // SPA fallback: serve index.html for unknown frontend routes
            if let Some(index) = Assets::get("index.html") {
                Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(Body::from(index.data))
                    .unwrap()
            } else {
                Response::builder()
                    .status(404)
                    .body(Body::from("Not Found"))
                    .unwrap()
            }
        }
    }
}
