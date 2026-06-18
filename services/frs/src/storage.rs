//! S3-compatible storage backend (MinIO).
//! Handles file upload, download, delete, and listing operations.

use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::{
    delete_object::DeleteObjectOutput, get_object::GetObjectOutput, head_object::HeadObjectOutput,
    list_objects_v2::ListObjectsV2Output, put_object::PutObjectOutput,
};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;
use tracing::instrument;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct StorageService {
    client: Client,
    bucket: String,
}

impl StorageService {
    /// Create a new S3 storage service connected to MinIO.
    pub async fn new(config: &AppConfig) -> Result<Self, anyhow::Error> {
        let creds = aws_credential_types::Credentials::new(
            &config.s3_access_key,
            &config.s3_secret_key,
            None,
            None,
            "minio",
        );

        let s3_config = aws_sdk_s3::config::Builder::new()
            .endpoint_url(&config.s3_endpoint)
            .region(aws_types::region::Region::new(config.s3_region.clone()))
            .credentials_provider(creds)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        // Ensure the bucket exists
        let service = Self {
            client,
            bucket: config.s3_bucket.clone(),
        };
        service.ensure_bucket().await?;

        Ok(service)
    }

    /// Create the bucket if it doesn't exist.
    async fn ensure_bucket(&self) -> Result<(), anyhow::Error> {
        match self.client.head_bucket().bucket(&self.bucket).send().await {
            Ok(_) => {
                tracing::info!("Bucket '{}' already exists", self.bucket);
                Ok(())
            }
            Err(SdkError::ServiceError(err)) if err.err().meta().code() == Some("NotFound") => {
                self.client
                    .create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await?;
                tracing::info!("Created bucket '{}'", self.bucket);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Could not verify bucket '{}': {}", self.bucket, e);
                // Try creating anyway
                self.client
                    .create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await?;
                tracing::info!("Created bucket '{}' after retry", self.bucket);
                Ok(())
            }
        }
    }

    /// Upload a file to MinIO.
    #[instrument(skip(self, data))]
    pub async fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: Option<&str>,
    ) -> Result<PutObjectOutput, anyhow::Error> {
        let body = ByteStream::from(data);
        let mut req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body);

        if let Some(ct) = content_type {
            req = req.content_type(ct);
        }

        let resp = req.send().await?;
        tracing::info!("Uploaded: s3://{}/{}", self.bucket, key);
        Ok(resp)
    }

    /// Download a file from MinIO.
    #[instrument(skip(self))]
    pub async fn download(&self, key: &str) -> Result<GetObjectOutput, anyhow::Error> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(resp)
    }

    /// Delete a file from MinIO.
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<DeleteObjectOutput, anyhow::Error> {
        let resp = self
            .client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        tracing::info!("Deleted: s3://{}/{}", self.bucket, key);
        Ok(resp)
    }

    /// Get file metadata (head).
    #[instrument(skip(self))]
    pub async fn head(&self, key: &str) -> Result<HeadObjectOutput, anyhow::Error> {
        let resp = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(resp)
    }

    /// List files with optional prefix.
    #[instrument(skip(self))]
    pub async fn list(
        &self,
        prefix: Option<&str>,
        max_keys: Option<i32>,
    ) -> Result<ListObjectsV2Output, anyhow::Error> {
        let mut req = self.client.list_objects_v2().bucket(&self.bucket);

        if let Some(p) = prefix {
            req = req.prefix(p);
        }
        if let Some(m) = max_keys {
            req = req.max_keys(m);
        }

        let resp = req.send().await?;
        Ok(resp)
    }

    /// Generate a presigned upload URL (valid for the given duration).
    pub async fn presigned_upload_url(
        &self,
        key: &str,
        expires_secs: u64,
    ) -> Result<String, anyhow::Error> {
        use aws_sdk_s3::presigning::PresigningConfig;
        use std::time::Duration;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(
                expires_secs,
            ))?)
            .await?;

        Ok(presigned.uri().to_string())
    }

    /// Generate a presigned download URL.
    pub async fn presigned_download_url(
        &self,
        key: &str,
        expires_secs: u64,
    ) -> Result<String, anyhow::Error> {
        use aws_sdk_s3::presigning::PresigningConfig;
        use std::time::Duration;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(
                expires_secs,
            ))?)
            .await?;

        Ok(presigned.uri().to_string())
    }
}
