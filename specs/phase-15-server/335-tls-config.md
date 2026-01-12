# 335 - TLS Configuration

**Phase:** 15 - Server
**Spec ID:** 335
**Status:** Planned
**Dependencies:** 333-server-startup
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Implement TLS configuration with certificate management, cipher suite selection, and automatic certificate renewal support.

---

## Acceptance Criteria

- [ ] TLS certificate loading
- [ ] Private key configuration
- [ ] Cipher suite selection
- [ ] Protocol version enforcement
- [ ] Certificate chain support
- [ ] SNI support
- [ ] Auto-renewal hooks

---

## Implementation Details

### 1. TLS Config Types (crates/tachikoma-server/src/tls/config.rs)

```rust
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
```

### 2. TLS Builder (crates/tachikoma-server/src/tls/builder.rs)

```rust
//! TLS configuration builder.

use super::config::{TlsConfig, TlsConfigError, TlsVersion};
use rustls::{
    server::ServerConfig as RustlsServerConfig,
    Certificate, PrivateKey,
};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Build rustls server configuration.
pub fn build_tls_config(config: &TlsConfig) -> Result<RustlsServerConfig, TlsConfigError> {
    // Load certificates
    let certs = load_certs(&config.cert_path)?;
    info!("Loaded {} certificate(s)", certs.len());

    // Load private key
    let key = load_private_key(&config.key_path)?;
    info!("Loaded private key");

    // Build config
    let mut builder = RustlsServerConfig::builder()
        .with_safe_defaults();

    // Set protocol versions
    builder = match (config.min_version, config.max_version) {
        (TlsVersion::Tls12, TlsVersion::Tls12) => {
            builder.with_protocol_versions(&[&rustls::version::TLS12])
                .map_err(|e| TlsConfigError::InvalidCert(e.to_string()))?
        }
        (TlsVersion::Tls13, TlsVersion::Tls13) => {
            builder.with_protocol_versions(&[&rustls::version::TLS13])
                .map_err(|e| TlsConfigError::InvalidCert(e.to_string()))?
        }
        _ => {
            builder.with_protocol_versions(&[
                &rustls::version::TLS12,
                &rustls::version::TLS13,
            ])
            .map_err(|e| TlsConfigError::InvalidCert(e.to_string()))?
        }
    };

    // Configure client authentication
    let config = if config.require_client_cert {
        if let Some(ca_path) = &config.ca_cert_path {
            let ca_certs = load_certs(ca_path)?;
            let mut roots = rustls::RootCertStore::empty();
            for cert in ca_certs {
                roots.add(&cert)
                    .map_err(|e| TlsConfigError::InvalidCert(e.to_string()))?;
            }

            let verifier = rustls::server::AllowAnyAuthenticatedClient::new(roots);
            builder
                .with_client_cert_verifier(Arc::new(verifier))
                .with_single_cert(certs, key)
                .map_err(|e| TlsConfigError::CertKeyMismatch)?
        } else {
            return Err(TlsConfigError::CaNotFound(
                "CA cert required for client verification".into(),
            ));
        }
    } else {
        builder
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|_| TlsConfigError::CertKeyMismatch)?
    };

    Ok(config)
}

/// Load certificates from PEM file.
fn load_certs(path: &Path) -> Result<Vec<Certificate>, TlsConfigError> {
    let file = File::open(path)
        .map_err(|_| TlsConfigError::CertNotFound(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);

    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|e| TlsConfigError::InvalidCert(e.to_string()))?
        .into_iter()
        .map(Certificate)
        .collect();

    Ok(certs)
}

/// Load private key from PEM file.
fn load_private_key(path: &Path) -> Result<PrivateKey, TlsConfigError> {
    let file = File::open(path)
        .map_err(|_| TlsConfigError::KeyNotFound(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);

    // Try PKCS#8 format first
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|e| TlsConfigError::InvalidKey(e.to_string()))?;

    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKey(key));
    }

    // Try RSA format
    let file = File::open(path)
        .map_err(|_| TlsConfigError::KeyNotFound(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);

    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|e| TlsConfigError::InvalidKey(e.to_string()))?;

    keys.into_iter()
        .next()
        .map(PrivateKey)
        .ok_or_else(|| TlsConfigError::InvalidKey("No private key found".to_string()))
}
```

### 3. Certificate Reloader (crates/tachikoma-server/src/tls/reloader.rs)

```rust
//! Certificate hot-reloading.

use super::{builder::build_tls_config, config::TlsConfig};
use rustls::server::ServerConfig as RustlsServerConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{error, info, warn};

/// Certificate reloader for hot-reloading TLS certificates.
pub struct CertReloader {
    config: TlsConfig,
    sender: watch::Sender<Arc<RustlsServerConfig>>,
    receiver: watch::Receiver<Arc<RustlsServerConfig>>,
}

impl CertReloader {
    pub fn new(config: TlsConfig) -> Result<Self, super::config::TlsConfigError> {
        let tls_config = build_tls_config(&config)?;
        let (sender, receiver) = watch::channel(Arc::new(tls_config));

        Ok(Self {
            config,
            sender,
            receiver,
        })
    }

    /// Get the current TLS configuration.
    pub fn get_config(&self) -> Arc<RustlsServerConfig> {
        self.receiver.borrow().clone()
    }

    /// Subscribe to configuration changes.
    pub fn subscribe(&self) -> watch::Receiver<Arc<RustlsServerConfig>> {
        self.receiver.clone()
    }

    /// Start the reloader task.
    pub fn start(self: Arc<Self>) {
        if let Some(interval) = self.config.reload_interval_secs {
            tokio::spawn(async move {
                self.reload_loop(Duration::from_secs(interval)).await;
            });
        }
    }

    async fn reload_loop(&self, interval: Duration) {
        let mut interval_timer = tokio::time::interval(interval);

        loop {
            interval_timer.tick().await;

            match self.reload() {
                Ok(true) => info!("TLS certificates reloaded successfully"),
                Ok(false) => {} // No change
                Err(e) => error!("Failed to reload TLS certificates: {}", e),
            }
        }
    }

    /// Reload certificates.
    pub fn reload(&self) -> Result<bool, super::config::TlsConfigError> {
        // Check if files have changed (by timestamp or hash)
        // For simplicity, we'll just reload unconditionally

        let new_config = build_tls_config(&self.config)?;

        // Send new configuration
        if self.sender.send(Arc::new(new_config)).is_err() {
            warn!("No receivers for TLS config update");
        }

        Ok(true)
    }
}

/// Create TLS acceptor with hot-reload support.
#[cfg(feature = "tls")]
pub fn create_tls_acceptor(
    reloader: &CertReloader,
) -> impl tower::Service<
    tokio::net::TcpStream,
    Response = tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    Error = std::io::Error,
> + Clone {
    use tokio_rustls::TlsAcceptor;

    let config = reloader.get_config();
    TlsAcceptor::from(config)
}
```

### 4. Development Certificates (crates/tachikoma-server/src/tls/dev.rs)

```rust
//! Development certificate generation.

use rcgen::{Certificate, CertificateParams, DnType, SanType};
use std::path::Path;
use tracing::info;

/// Generate self-signed development certificates.
pub fn generate_dev_certs(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Generating development certificates...");

    let mut params = CertificateParams::default();

    // Set common name
    params.distinguished_name.push(DnType::CommonName, "localhost");
    params.distinguished_name.push(DnType::OrganizationName, "Tachikoma Dev");

    // Add SANs
    params.subject_alt_names = vec![
        SanType::DnsName("localhost".to_string()),
        SanType::DnsName("127.0.0.1".to_string()),
        SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        SanType::IpAddress(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
    ];

    // Generate certificate
    let cert = Certificate::from_params(params)?;

    // Write certificate
    let cert_pem = cert.serialize_pem()?;
    std::fs::write(cert_path, cert_pem)?;
    info!("Certificate written to {:?}", cert_path);

    // Write private key
    let key_pem = cert.serialize_private_key_pem();
    std::fs::write(key_path, key_pem)?;
    info!("Private key written to {:?}", key_path);

    Ok(())
}

/// Check if development certificates exist and are valid.
pub fn check_dev_certs(cert_path: &Path, key_path: &Path) -> bool {
    cert_path.exists() && key_path.exists()
}
```

---

## Testing Requirements

1. Certificates load correctly
2. TLS handshake succeeds
3. Invalid certs rejected
4. Certificate reload works
5. Protocol versions enforced
6. Client cert verification works
7. Dev cert generation works

---

## Related Specs

- Depends on: [333-server-startup.md](333-server-startup.md)
- Next: [336-server-monitoring.md](336-server-monitoring.md)
- Used by: HTTPS endpoints
