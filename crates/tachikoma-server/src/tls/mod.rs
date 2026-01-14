//! TLS configuration and certificate management.
//!
//! This module provides:
//! - TLS configuration types
//! - Certificate loading from PEM files
//! - Certificate chain support
//! - Private key loading (PKCS#8 and RSA formats)
//! - Protocol version enforcement
//! - Cipher suite selection
//! - Certificate hot-reloading
//! - Development certificate generation

pub mod builder;
pub mod config;
pub mod dev;
pub mod reloader;

pub use builder::build_tls_config;
pub use config::{TlsConfig, TlsConfigError, TlsVersion};
pub use reloader::{CertReloader, create_tls_acceptor};

#[cfg(feature = "dev-certs")]
pub use dev::{generate_dev_certs, check_dev_certs};