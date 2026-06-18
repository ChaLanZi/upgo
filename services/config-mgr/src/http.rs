//! HTTP REST handlers for configuration management.
//! Proxied by Gateway at /api/config/*

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::rnacos_client::{ConfigKey, RnacosClient};

pub fn routes(rnacos: Arc<RnacosClient>) -> Router {
    Router::new()
        .route("/api/config/get", get(get_config_handler))
        .route("/api/config/publish", post(publish_config_handler))
        .route("/api/config/delete", delete(delete_config_handler))
        .route("/api/config/list/{group}", get(list_configs_handler))
        .route("/api/config/watch", post(watch_configs_handler))
        .route("/api/config/cache/preload", post(preload_cache_handler))
        .with_state(rnacos)
}

// ── Request / Response types ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetConfigQuery {
    pub data_id: String,
    pub group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetConfigResponse {
    pub data_id: String,
    pub group: String,
    pub content: String,
    pub cached: bool,
}

#[derive(Debug, Deserialize)]
pub struct PublishConfigRequest {
    pub data_id: String,
    pub group: Option<String>,
    pub content: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishConfigResponse {
    pub success: bool,
    pub data_id: String,
    pub group: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteConfigQuery {
    pub data_id: String,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WatchRequest {
    pub data_ids: Vec<WatchItem>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WatchItem {
    pub data_id: String,
    pub group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WatchResponse {
    pub changed: Vec<WatchItem>,
}

#[derive(Debug, Deserialize)]
pub struct PreloadRequest {
    pub configs: Vec<PreloadItem>,
}

#[derive(Debug, Deserialize)]
pub struct PreloadItem {
    pub data_id: String,
    pub group: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigListResponse {
    pub configs: Vec<crate::rnacos_client::ConfigSummary>,
}

// ── Handlers ────────────────────────────────────────────

async fn get_config_handler(
    State(rnacos): State<Arc<RnacosClient>>,
    Query(query): Query<GetConfigQuery>,
) -> Result<Json<GetConfigResponse>, (StatusCode, String)> {
    let group = query.group.as_deref().unwrap_or("DEFAULT_GROUP");

    // Try cache first
    if let Some(cached) = rnacos.get_cached(&query.data_id, group).await {
        return Ok(Json(GetConfigResponse {
            data_id: query.data_id,
            group: group.to_string(),
            content: cached,
            cached: true,
        }));
    }

    // Fetch from RNacos
    let content = rnacos.get_config(&query.data_id, group).await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, format!("RNacos error: {}", e))
    })?;

    Ok(Json(GetConfigResponse {
        data_id: query.data_id,
        group: group.to_string(),
        content,
        cached: false,
    }))
}

async fn publish_config_handler(
    State(rnacos): State<Arc<RnacosClient>>,
    Json(req): Json<PublishConfigRequest>,
) -> Result<Json<PublishConfigResponse>, (StatusCode, String)> {
    let group = req.group.as_deref().unwrap_or("DEFAULT_GROUP");

    let success = rnacos
        .publish_config(&req.data_id, group, &req.content, req.content_type.as_deref())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Publish failed: {}", e)))?;

    Ok(Json(PublishConfigResponse {
        success,
        data_id: req.data_id,
        group: group.to_string(),
    }))
}

async fn delete_config_handler(
    State(rnacos): State<Arc<RnacosClient>>,
    Query(query): Query<DeleteConfigQuery>,
) -> Result<Json<PublishConfigResponse>, (StatusCode, String)> {
    let group = query.group.as_deref().unwrap_or("DEFAULT_GROUP");

    let success = rnacos
        .delete_config(&query.data_id, group)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Delete failed: {}", e)))?;

    Ok(Json(PublishConfigResponse {
        success,
        data_id: query.data_id,
        group: group.to_string(),
    }))
}

async fn list_configs_handler(
    State(rnacos): State<Arc<RnacosClient>>,
    Path(group): Path<String>,
) -> Result<Json<ConfigListResponse>, (StatusCode, String)> {
    let configs = rnacos
        .list_by_group(&group)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("List failed: {}", e)))?;

    Ok(Json(ConfigListResponse { configs }))
}

async fn watch_configs_handler(
    State(rnacos): State<Arc<RnacosClient>>,
    Json(req): Json<WatchRequest>,
) -> Result<Json<WatchResponse>, (StatusCode, String)> {
    let timeout = req.timeout_ms.unwrap_or(30000);

    let watched: Vec<ConfigKey> = req
        .data_ids
        .iter()
        .map(|item| ConfigKey {
            data_id: item.data_id.clone(),
            group: item.group.clone().unwrap_or_else(|| "DEFAULT_GROUP".to_string()),
        })
        .collect();

    let changed = rnacos
        .listen(&watched, timeout)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Watch failed: {}", e)))?;

    Ok(Json(WatchResponse {
        changed: changed
            .into_iter()
            .map(|k| WatchItem {
                data_id: k.data_id,
                group: Some(k.group),
            })
            .collect(),
    }))
}

async fn preload_cache_handler(
    State(rnacos): State<Arc<RnacosClient>>,
    Json(req): Json<PreloadRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let keys: Vec<ConfigKey> = req
        .configs
        .iter()
        .map(|item| ConfigKey {
            data_id: item.data_id.clone(),
            group: item.group.clone().unwrap_or_else(|| "DEFAULT_GROUP".to_string()),
        })
        .collect();

    let count = keys.len();
    rnacos.preload(&keys).await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, format!("Preload failed: {}", e))
    })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "preloaded": count,
    })))
}
