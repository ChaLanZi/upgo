//! API reverse proxy handler.
//! Forwards `/api/*` requests to the auth backend.

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use http_body_util::BodyExt;

/// Proxy an API request to the auth backend.
/// Strips `/api` prefix and forwards the rest.
pub async fn api_proxy(req: axum::extract::Request<Body>, backend: &str) -> Response<Body> {
    let client = reqwest::Client::new();

    let method = req.method().clone();
    let uri_path = req.uri().path().to_string();
    let query = req
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();
    let path = uri_path
        .strip_prefix("/api")
        .unwrap_or(&uri_path)
        .to_string();
    let upstream_url = format!("{}{}{}", backend, path, query);
    let headers = req.headers().clone();

    tracing::debug!("Proxying {} {} → {}", method, uri_path, upstream_url);

    let body_bytes = req
        .collect()
        .await
        .map(|b| b.to_bytes().to_vec())
        .unwrap_or_default();

    let proxied = client
        .request(method, &upstream_url)
        .headers(headers)
        .body(body_bytes);

    match proxied.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let body = Body::from(resp.bytes().await.unwrap_or_default());

            let mut response = Response::new(body);
            *response.status_mut() = status;
            for (key, value) in headers.iter() {
                if key.as_str() != "host" && key.as_str() != "transfer-encoding" {
                    response.headers_mut().insert(key, value.clone());
                }
            }
            response
        }
        Err(e) => {
            tracing::error!("Proxy error: {}", e);
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Bad Gateway: {}", e)))
                .unwrap()
        }
    }
}
