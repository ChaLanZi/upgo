use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: String,
    pub log_level: String,
    pub rnacos_addr: String,
    pub rnacos_namespace: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:9095".to_string()),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            rnacos_addr: std::env::var("RNACOS_ADDR")
                .unwrap_or_else(|_| "http://rnacos:8848".to_string()),
            rnacos_namespace: std::env::var("RNACOS_NAMESPACE")
                .unwrap_or_else(|_| "public".to_string()),
        })
    }
}
