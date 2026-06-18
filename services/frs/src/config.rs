use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub grpc_addr: String,
    pub http_addr: String,
    pub log_level: String,

    // MinIO / S3-compatible storage
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_bucket: String,

    // NATS events
    pub nats_url: String,
    pub enable_nats: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            grpc_addr: std::env::var("GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50054".to_string()),
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:9094".to_string()),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            s3_endpoint: std::env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://minio.upgo.svc.cluster.local:9000".to_string()),
            s3_region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            s3_access_key: std::env::var("S3_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            s3_secret_key: std::env::var("S3_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            s3_bucket: std::env::var("S3_BUCKET").unwrap_or_else(|_| "upgo-files".to_string()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://nats.upgo.svc.cluster.local:4222".to_string()),
            enable_nats: std::env::var("ENABLE_NATS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        })
    }
}
