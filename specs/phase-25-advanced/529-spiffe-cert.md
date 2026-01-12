# Spec 529: SPIFFE Certificate Management

## Overview
Certificate management system for SPIFFE SVIDs including issuance, rotation, revocation, and trust bundle distribution for Tachikoma's autonomous infrastructure.

## Requirements

### Certificate Authority Integration
```go
type CertificateAuthority interface {
    // Issue new SVID
    IssueSVID(ctx context.Context, req SVIDRequest) (*X509SVID, error)

    // Renew existing SVID
    RenewSVID(ctx context.Context, current *X509SVID) (*X509SVID, error)

    // Revoke SVID
    RevokeSVID(ctx context.Context, serialNumber string) error

    // Get trust bundle
    GetBundle(ctx context.Context) (*x509bundle.Bundle, error)
}
```

### SVID Request Structure
```go
type SVIDRequest struct {
    SPIFFEID     string        `json:"spiffeId"`
    CSR          []byte        `json:"csr,omitempty"`
    TTL          time.Duration `json:"ttl"`
    DNSNames     []string      `json:"dnsNames,omitempty"`
    IPAddresses  []net.IP      `json:"ipAddresses,omitempty"`
}
```

### Certificate Lifecycle
```go
type CertificateLifecycle struct {
    IssuedAt     time.Time     `json:"issuedAt"`
    ExpiresAt    time.Time     `json:"expiresAt"`
    RenewAfter   time.Time     `json:"renewAfter"`
    TTL          time.Duration `json:"ttl"`
    SerialNumber string        `json:"serialNumber"`
    Revoked      bool          `json:"revoked"`
}
```

### Automatic Rotation
- Proactive renewal before expiry
- Configurable renewal window (default: 50% of TTL)
- Graceful certificate handoff
- Zero-downtime rotation
- Fallback to previous cert on failure

### Rotation Configuration
```go
type RotationConfig struct {
    Enabled         bool          `json:"enabled"`
    RenewalPercent  int           `json:"renewalPercent"` // % of TTL remaining
    RetryInterval   time.Duration `json:"retryInterval"`
    MaxRetries      int           `json:"maxRetries"`
    GracePeriod     time.Duration `json:"gracePeriod"`
}
```

### Trust Bundle Management
```go
type BundleManager interface {
    // Update local bundle
    UpdateBundle(ctx context.Context, bundle *x509bundle.Bundle) error

    // Fetch federated bundle
    FetchFederatedBundle(ctx context.Context, trustDomain string) (*x509bundle.Bundle, error)

    // Watch for bundle updates
    WatchBundle(ctx context.Context) (<-chan *BundleUpdate, error)
}
```

### Certificate Storage
- In-memory secure storage
- Encrypted on-disk backup
- Secret backend integration (Vault)
- Key material protection
- HSM support (optional)

### Revocation Checking
- CRL distribution points
- OCSP stapling
- Certificate transparency logs
- Revocation cache
- Fail-open vs fail-closed policies

### Audit Logging
- Certificate issuance events
- Rotation events
- Revocation events
- Trust bundle updates
- Failed authentication attempts

## Dependencies
- Spec 528: SPIFFE Identity

## Verification
- [ ] Certificate issuance works
- [ ] Auto-rotation functional
- [ ] Revocation checking works
- [ ] Bundle distribution correct
- [ ] Audit logs complete
