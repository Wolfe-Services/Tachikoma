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