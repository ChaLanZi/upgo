//! HTTP REST handlers for file operations.
//! These endpoints are proxied by the gateway at /api/files/*.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::StorageService;

pub fn routes(storage: Arc<StorageService>) -> Router {
    Router::new()
        .route("/api/files/upload", post(upload_handler))
        .route("/api/files/download/{key}", get(download_handler))
        .route("/api/files/delete/{key}", delete(delete_handler))
        .route("/api/files/info/{key}", get(info_handler))
        .route("/api/files/list", get(list_handler))
        .route(
            "/api/files/presigned/upload",
            post(presigned_upload_handler),
        )
        .route(
            "/api/files/presigned/download/{key}",
            get(presigned_download_handler),
        )
        .with_state(storage)
}

// ── Request / Response types ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub key: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub key: String,
    pub etag: String,
    pub size: usize,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub key: String,
    pub size: i64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileListItem>,
    pub is_truncated: bool,
}

#[derive(Debug, Serialize)]
pub struct FileListItem {
    pub key: String,
    pub size: i64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub prefix: Option<String>,
    pub max_keys: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PresignedResponse {
    pub url: String,
    pub key: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct PresignedUploadRequest {
    pub key: String,
    pub expires_secs: Option<u64>,
}

// ── Handlers ────────────────────────────────────────────

async fn upload_handler(
    State(storage): State<Arc<StorageService>>,
    axum::extract::Query(req): axum::extract::Query<UploadRequest>,
    body: Bytes,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    let key = if req.key.is_empty() {
        Uuid::now_v7().to_string()
    } else {
        req.key
    };

    let resp = storage
        .upload(&key, body, req.content_type.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Upload failed: {}", e),
            )
        })?;

    Ok(Json(UploadResponse {
        key,
        etag: resp.e_tag.unwrap_or_default().trim_matches('"').to_string(),
        size: 0,
    }))
}

async fn download_handler(
    State(storage): State<Arc<StorageService>>,
    Path(key): Path<String>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let resp = storage.download(&key).await.map_err(|e| {
        if e.to_string().contains("NoSuchKey") {
            (StatusCode::NOT_FOUND, "File not found".to_string())
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Download failed: {}", e),
            )
        }
    })?;

    let content_type = resp
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
    let content_length = resp.content_length().unwrap_or(0);

    // Read all bytes from the stream
    let data = resp.body.collect().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read file: {}", e),
        )
    })?;
    let body = Body::from(data.into_bytes().to_vec());
    let mut response = Response::new(body);
    response
        .headers_mut()
        .insert("Content-Type", content_type.parse().unwrap());
    response.headers_mut().insert(
        "Content-Length",
        content_length.to_string().parse().unwrap(),
    );
    Ok(response)
}

async fn delete_handler(
    State(storage): State<Arc<StorageService>>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    storage.delete(&key).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Delete failed: {}", e),
        )
    })?;

    Ok(Json(serde_json::json!({ "status": "deleted", "key": key })))
}

async fn info_handler(
    State(storage): State<Arc<StorageService>>,
    Path(key): Path<String>,
) -> Result<Json<FileInfo>, (StatusCode, String)> {
    let resp = storage.head(&key).await.map_err(|e| {
        if e.to_string().contains("NoSuchKey") || e.to_string().contains("Not Found") {
            (StatusCode::NOT_FOUND, "File not found".to_string())
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Info failed: {}", e),
            )
        }
    })?;

    Ok(Json(FileInfo {
        key,
        size: resp.content_length().unwrap_or(0),
        content_type: resp.content_type().map(|s| s.to_string()),
        etag: resp.e_tag().map(|s| s.trim_matches('"').to_string()),
        last_modified: resp.last_modified().map(|d| d.to_string()),
    }))
}

async fn list_handler(
    State(storage): State<Arc<StorageService>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<FileListResponse>, (StatusCode, String)> {
    let resp = storage
        .list(query.prefix.as_deref(), query.max_keys)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("List failed: {}", e),
            )
        })?;

    let files = resp
        .contents()
        .iter()
        .map(|obj| FileListItem {
            key: obj.key().unwrap_or("").to_string(),
            size: obj.size().unwrap_or(0),
            etag: obj.e_tag().map(|s| s.trim_matches('"').to_string()),
            last_modified: obj.last_modified().map(|d| d.to_string()),
        })
        .collect();

    Ok(Json(FileListResponse {
        files,
        is_truncated: resp.is_truncated().unwrap_or(false),
    }))
}

async fn presigned_upload_handler(
    State(storage): State<Arc<StorageService>>,
    Json(req): Json<PresignedUploadRequest>,
) -> Result<Json<PresignedResponse>, (StatusCode, String)> {
    let expires = req.expires_secs.unwrap_or(3600);
    let key = if req.key.is_empty() {
        Uuid::now_v7().to_string()
    } else {
        req.key
    };

    let url = storage
        .presigned_upload_url(&key, expires)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Presigned URL failed: {}", e),
            )
        })?;

    Ok(Json(PresignedResponse {
        url,
        key,
        expires_in: expires,
    }))
}

async fn presigned_download_handler(
    State(storage): State<Arc<StorageService>>,
    Path(key): Path<String>,
    Query(query): Query<PresignedDownloadQuery>,
) -> Result<Json<PresignedResponse>, (StatusCode, String)> {
    let expires = query.expires_secs.unwrap_or(3600);

    let url = storage
        .presigned_download_url(&key, expires)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Presigned URL failed: {}", e),
            )
        })?;

    Ok(Json(PresignedResponse {
        url,
        key,
        expires_in: expires,
    }))
}

#[derive(Debug, Deserialize)]
pub struct PresignedDownloadQuery {
    pub expires_secs: Option<u64>,
}
