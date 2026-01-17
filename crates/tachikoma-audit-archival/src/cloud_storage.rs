//! Cloud storage support for archives.

use crate::archive::{ArchiveLocation, ArchiveMetadata};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Cloud storage provider trait.
#[async_trait]
pub trait CloudStorageProvider {
    /// Upload archive to cloud storage.
    async fn upload(
        &self,
        local_path: &Path,
        location: &ArchiveLocation,
    ) -> Result<(), CloudStorageError>;

    /// Download archive from cloud storage.
    async fn download(
        &self,
        location: &ArchiveLocation,
        local_path: &Path,
    ) -> Result<(), CloudStorageError>;

    /// Delete archive from cloud storage.
    async fn delete(&self, location: &ArchiveLocation) -> Result<(), CloudStorageError>;

    /// Check if archive exists in cloud storage.
    async fn exists(&self, location: &ArchiveLocation) -> Result<bool, CloudStorageError>;
}

/// Cloud storage manager.
pub struct CloudStorageManager {
    providers: std::collections::HashMap<String, Box<dyn CloudStorageProvider + Send + Sync>>,
}

impl CloudStorageManager {
    /// Create a new cloud storage manager.
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a storage provider.
    pub fn register_provider(
        &mut self,
        name: String,
        provider: Box<dyn CloudStorageProvider + Send + Sync>,
    ) {
        self.providers.insert(name, provider);
    }

    /// Upload archive to cloud storage.
    pub async fn upload(
        &self,
        local_path: &Path,
        location: &ArchiveLocation,
        metadata: &ArchiveMetadata,
    ) -> Result<(), CloudStorageError> {
        match location {
            ArchiveLocation::Local { .. } => {
                // Already local, nothing to do
                Ok(())
            }
            ArchiveLocation::S3 { .. } => {
                if let Some(provider) = self.providers.get("s3") {
                    provider.upload(local_path, location).await
                } else {
                    Err(CloudStorageError::ProviderNotAvailable("s3".to_string()))
                }
            }
            ArchiveLocation::AzureBlob { .. } => {
                if let Some(provider) = self.providers.get("azure") {
                    provider.upload(local_path, location).await
                } else {
                    Err(CloudStorageError::ProviderNotAvailable("azure".to_string()))
                }
            }
            ArchiveLocation::Gcs { .. } => {
                if let Some(provider) = self.providers.get("gcs") {
                    provider.upload(local_path, location).await
                } else {
                    Err(CloudStorageError::ProviderNotAvailable("gcs".to_string()))
                }
            }
        }
    }

    /// Download archive from cloud storage.
    pub async fn download(
        &self,
        location: &ArchiveLocation,
        local_path: &Path,
    ) -> Result<(), CloudStorageError> {
        match location {
            ArchiveLocation::Local { path } => {
                // Copy from local path to target path
                tokio::fs::copy(path, local_path).await?;
                Ok(())
            }
            ArchiveLocation::S3 { .. } => {
                if let Some(provider) = self.providers.get("s3") {
                    provider.download(location, local_path).await
                } else {
                    Err(CloudStorageError::ProviderNotAvailable("s3".to_string()))
                }
            }
            ArchiveLocation::AzureBlob { .. } => {
                if let Some(provider) = self.providers.get("azure") {
                    provider.download(location, local_path).await
                } else {
                    Err(CloudStorageError::ProviderNotAvailable("azure".to_string()))
                }
            }
            ArchiveLocation::Gcs { .. } => {
                if let Some(provider) = self.providers.get("gcs") {
                    provider.download(location, local_path).await
                } else {
                    Err(CloudStorageError::ProviderNotAvailable("gcs".to_string()))
                }
            }
        }
    }
}

impl Default for CloudStorageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock cloud storage provider for testing.
pub struct MockCloudStorageProvider;

#[async_trait]
impl CloudStorageProvider for MockCloudStorageProvider {
    async fn upload(
        &self,
        local_path: &Path,
        _location: &ArchiveLocation,
    ) -> Result<(), CloudStorageError> {
        // For testing, just verify the file exists
        if !local_path.exists() {
            return Err(CloudStorageError::FileNotFound(local_path.to_path_buf()));
        }
        Ok(())
    }

    async fn download(
        &self,
        _location: &ArchiveLocation,
        local_path: &Path,
    ) -> Result<(), CloudStorageError> {
        // For testing, create an empty file
        let mut file = File::create(local_path).await?;
        file.write_all(b"mock archive data\n").await?;
        Ok(())
    }

    async fn delete(&self, _location: &ArchiveLocation) -> Result<(), CloudStorageError> {
        Ok(())
    }

    async fn exists(&self, _location: &ArchiveLocation) -> Result<bool, CloudStorageError> {
        Ok(true)
    }
}

/// Cloud storage errors.
#[derive(Debug, thiserror::Error)]
pub enum CloudStorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("provider not available: {0}")]
    ProviderNotAvailable(String),
    #[error("file not found: {0}")]
    FileNotFound(std::path::PathBuf),
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("network error: {0}")]
    Network(String),
}