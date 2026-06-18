//! RNacos client — wraps Nacos v1 HTTP API for configuration management.
//!
//! Supports:
//! - CRUD: get / publish / delete configs
//! - Watch: long-polling listener for config changes (push)
//! - Caching: optional local cache for frequently accessed configs

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::instrument;

use crate::config::AppConfig;

/// A configuration item stored in RNacos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub data_id: String,
    pub group: String,
    pub content: String,
    #[serde(rename = "contentType", default)]
    pub content_type: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub version: i64,
}

/// Summary of a config item (list response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    pub data_id: String,
    pub group: String,
    pub version: i64,
}

/// RNacos client with optional local cache.
pub struct RnacosClient {
    client: Client,
    base_url: String,
    namespace: String,
    /// Local cache: (data_id, group) -> content
    cache: Arc<RwLock<HashMap<(String, String), String>>>,
}

impl RnacosClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/nacos/v1/cs", config.rnacos_addr),
            namespace: config.rnacos_namespace.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// ── CRUD Operations ──────────────────────────────────

    /// Get a configuration value.
    #[instrument(skip(self))]
    pub async fn get_config(&self, data_id: &str, group: &str) -> Result<String> {
        let url = format!(
            "{}/configs?dataId={}&group={}&tenant={}",
            self.base_url,
            urlencoding(data_id),
            urlencoding(group),
            self.namespace,
        );

        let resp = self.client.get(&url).send().await?;
        let content = resp.text().await?;

        // Update cache
        self.cache
            .write()
            .await
            .insert((data_id.to_string(), group.to_string()), content.clone());

        Ok(content)
    }

    /// Publish / update a configuration.
    #[instrument(skip(self))]
    pub async fn publish_config(
        &self,
        data_id: &str,
        group: &str,
        content: &str,
        content_type: Option<&str>,
    ) -> Result<bool> {
        let url = format!("{}/configs", self.base_url);

        let mut form = HashMap::new();
        form.insert("dataId", data_id);
        form.insert("group", group);
        form.insert("content", content);
        form.insert("tenant", &self.namespace);
        if let Some(ct) = content_type {
            form.insert("type", ct);
        }

        let resp = self.client.post(&url).form(&form).send().await?;
        let body = resp.text().await?;

        // Update cache
        self.cache
            .write()
            .await
            .insert((data_id.to_string(), group.to_string()), content.to_string());

        tracing::info!(data_id, group, "Config published");
        Ok(body.contains("true"))
    }

    /// Delete a configuration.
    #[instrument(skip(self))]
    pub async fn delete_config(&self, data_id: &str, group: &str) -> Result<bool> {
        let url = format!(
            "{}/configs?dataId={}&group={}&tenant={}",
            self.base_url,
            urlencoding(data_id),
            urlencoding(group),
            self.namespace,
        );

        let resp = self.client.delete(&url).send().await?;
        let body = resp.text().await?;

        // Remove from cache
        self.cache
            .write()
            .await
            .remove(&(data_id.to_string(), group.to_string()));

        tracing::info!(data_id, group, "Config deleted");
        Ok(body.contains("true"))
    }

    /// ── Watch / Listen ───────────────────────────────────

    /// Listen for config changes using Nacos long-polling.
    /// Returns changed (data_id, group) pairs, or empty vec on timeout.
    pub async fn listen(
        &self,
        watched: &[ConfigKey],
        timeout_ms: u64,
    ) -> Result<Vec<ConfigKey>> {
        let url = format!("{}/configs/listener", self.base_url);

        // Build listening configs string: data_id[[group[[tenant]]]
        let listening_configs: Vec<String> = watched
            .iter()
            .map(|k| format!("{}[[{}[[{}]]]", k.data_id, k.group, self.namespace))
            .collect();

        let probe_modify_request = listening_configs.join("\x02");

        let resp = self
            .client
            .post(&url)
            .header("Long-Pulling-Timeout", timeout_ms.to_string())
            .form(&[("Listening-Configs", &probe_modify_request)])
            .send()
            .await?;

        let body = resp.text().await?;
        if body.is_empty() {
            return Ok(Vec::new());
        }

        // Parse changed configs
        let changed: Vec<ConfigKey> = body
            .split('\x02')
            .filter(|s| !s.is_empty())
            .filter_map(|s| {
                let parts: Vec<&str> = s.split("[[").collect();
                if parts.len() >= 2 {
                    Some(ConfigKey {
                        data_id: parts[0].to_string(),
                        group: parts[1].trim_end_matches("]]").to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Invalidate cache for changed items
        {
            let mut cache = self.cache.write().await;
            for key in &changed {
                cache.remove(&(key.data_id.clone(), key.group.clone()));
            }
        }

        Ok(changed)
    }

    /// ── Cache Operations ─────────────────────────────────

    /// Get a config from local cache (if available).
    pub async fn get_cached(&self, data_id: &str, group: &str) -> Option<String> {
        self.cache
            .read()
            .await
            .get(&(data_id.to_string(), group.to_string()))
            .cloned()
    }

    /// Pre-load multiple configs into cache.
    pub async fn preload(&self, configs: &[ConfigKey]) -> Result<()> {
        for key in configs {
            match self.get_config(&key.data_id, &key.group).await {
                Ok(content) => {
                    tracing::info!(data_id = %key.data_id, "Pre-loaded config");
                    self.cache
                        .write()
                        .await
                        .insert((key.data_id.clone(), key.group.clone()), content);
                }
                Err(e) => {
                    tracing::warn!(
                        data_id = %key.data_id,
                        "Failed to pre-load config: {}",
                        e
                    );
                }
            }
        }
        Ok(())
    }

    /// List all configs by group.
    pub async fn list_by_group(&self, group: &str) -> Result<Vec<ConfigSummary>> {
        let url = format!(
            "{}/configs?group={}&tenant={}",
            self.base_url,
            urlencoding(group),
            self.namespace,
        );

        let resp = self.client.get(&url).send().await?;
        let data: serde_json::Value = resp.json().await?;
        let mut items = Vec::new();

        if let Some(pages) = data["pages"].as_array() {
            for page in pages {
                if let Some(page_items) = page.as_array() {
                    for item in page_items {
                        items.push(ConfigSummary {
                            data_id: item["dataId"].as_str().unwrap_or("").to_string(),
                            group: item["group"].as_str().unwrap_or("").to_string(),
                            version: item["version"].as_i64().unwrap_or(0),
                        });
                    }
                }
            }
        }

        Ok(items)
    }
}

/// Key identifier for a configuration.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigKey {
    pub data_id: String,
    pub group: String,
}

/// Simple URL encoding (avoids adding url crate dependency).
fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('/', "%2F")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('?', "%3F")
}
