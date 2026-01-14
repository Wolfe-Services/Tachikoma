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