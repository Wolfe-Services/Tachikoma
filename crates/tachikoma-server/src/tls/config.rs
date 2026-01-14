//! TLS configuration types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS.
    pub enabled: bool,
    /// Certificate file path (PEM format).
    pub cert_path: PathBuf,
    /// Private key file path (PEM format).
    pub key_path: PathBuf,
    /// CA certificate path for client verification.
    pub ca_cert_path: Option<PathBuf>,
    /// Minimum TLS version.
    #[serde(default = "default_min_version")]
    pub min_version: TlsVersion,
    /// Maximum TLS version.
    #[serde(default = "default_max_version")]
    pub max_version: TlsVersion,
    /// Require client certificate.
    #[serde(default)]
    pub require_client_cert: bool,
    /// Allowed cipher suites.
    #[serde(default)]
    pub cipher_suites: Vec<String>,
    /// Enable OCSP stapling.
    #[serde(default)]
    pub ocsp_stapling: bool,
    /// Certificate reload interval (seconds).
    pub reload_interval_secs: Option<u64>,
}

fn default_min_version() -> TlsVersion {
    TlsVersion::Tls12
}

fn default_max_version() -> TlsVersion {
    TlsVersion::Tls13
}

/// TLS protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    #[serde(rename = "1.2")]
    Tls12,
    #[serde(rename = "1.3")]
    Tls13,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: PathBuf::from("certs/server.crt"),
            key_path: PathBuf::from("certs/server.key"),
            ca_cert_path: None,
            min_version: TlsVersion::Tls12,
            max_version: TlsVersion::Tls13,
            require_client_cert: false,
            cipher_suites: Vec::new(),
            ocsp_stapling: false,
            reload_interval_secs: None,
        }
    }
}

impl TlsConfig {
    /// Create config for development (self-signed).
    pub fn development() -> Self {
        Self {
            enabled: true,
            cert_path: PathBuf::from("certs/dev.crt"),
            key_path: PathBuf::from("certs/dev.key"),
            min_version: TlsVersion::Tls12,
            ..Default::default()
        }
    }

    /// Create config for production.
    pub fn production(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            enabled: true,
            cert_path,
            key_path,
            min_version: TlsVersion::Tls12,
            max_version: TlsVersion::Tls13,
            ocsp_stapling: true,
            reload_interval_secs: Some(3600), // 1 hour
            ..Default::default()
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), TlsConfigError> {
        if !self.enabled {
            return Ok(());
        }

        if !self.cert_path.exists() {
            return Err(TlsConfigError::CertNotFound(self.cert_path.clone()));
        }

        if !self.key_path.exists() {
            return Err(TlsConfigError::KeyNotFound(self.key_path.clone()));
        }

        if let Some(ca_path) = &self.ca_cert_path {
            if !ca_path.exists() {
                return Err(TlsConfigError::CaNotFound(ca_path.clone()));
            }
        }

        Ok(())
    }
}

/// TLS configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum TlsConfigError {
    #[error("Certificate file not found: {0}")]
    CertNotFound(PathBuf),

    #[error("Private key file not found: {0}")]
    KeyNotFound(PathBuf),

    #[error("CA certificate file not found: {0}")]
    CaNotFound(PathBuf),

    #[error("Invalid certificate: {0}")]
    InvalidCert(String),

    #[error("Invalid private key: {0}")]
    InvalidKey(String),

    #[error("Certificate/key mismatch")]
    CertKeyMismatch,
}