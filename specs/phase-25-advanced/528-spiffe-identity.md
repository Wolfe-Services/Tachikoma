# Spec 528: SPIFFE Identity Management

## Overview
SPIFFE (Secure Production Identity Framework for Everyone) integration for cryptographic workload identity, enabling secure service-to-service authentication for Tachikoma agents.

## Requirements

### SPIFFE ID Structure
```
spiffe://tachikoma.local/agent/<agent-id>
spiffe://tachikoma.local/pod/<namespace>/<pod-name>
spiffe://tachikoma.local/service/<service-name>
```

### Identity Types
```go
// SPIFFEIdentity represents a workload identity
type SPIFFEIdentity struct {
    TrustDomain string    `json:"trustDomain"`
    Path        string    `json:"path"`
    SVID        *X509SVID `json:"svid"`
    ExpiresAt   time.Time `json:"expiresAt"`
    Renewable   bool      `json:"renewable"`
}

// X509SVID represents an X.509 SPIFFE Verifiable Identity Document
type X509SVID struct {
    Certificate *x509.Certificate
    PrivateKey  crypto.PrivateKey
    Bundle      *x509bundle.Bundle
}
```

### Workload API Client
```go
type WorkloadAPIClient interface {
    // Fetch current SVID
    FetchX509SVID(ctx context.Context) (*X509SVID, error)

    // Watch for SVID updates
    WatchX509SVID(ctx context.Context) (<-chan *X509SVIDUpdate, error)

    // Fetch trust bundle
    FetchX509Bundle(ctx context.Context) (*x509bundle.Bundle, error)

    // Validate peer identity
    ValidatePeerIdentity(ctx context.Context, peer *X509SVID) error
}
```

### SPIRE Integration
- SPIRE server connectivity
- SPIRE agent sidecar
- Attestation mechanisms
- Node attestation
- Workload attestation

### Identity Lifecycle
1. Agent registration with SPIRE
2. Initial SVID issuance
3. Automatic SVID rotation
4. Trust bundle updates
5. Graceful credential refresh

### Trust Domains
```go
type TrustDomain struct {
    Name          string   `json:"name"`
    BundleEndpoint string  `json:"bundleEndpoint"`
    FederatedWith []string `json:"federatedWith"`
}
```

### Federation
- Cross-cluster federation
- Multi-cloud identity
- External service authentication
- Bundle exchange protocol
- Trust verification

### mTLS Configuration
```go
type MTLSConfig struct {
    Enabled        bool     `json:"enabled"`
    AllowedSPIFFEIDs []string `json:"allowedSpiffeIds"`
    TrustDomains   []string `json:"trustDomains"`
    MinTLSVersion  string   `json:"minTlsVersion"`
}
```

### Attestation Policies
- Kubernetes attestor
- Docker attestor
- Unix attestor
- AWS/GCP/Azure attestors
- Custom attestors

## Dependencies
- None (foundational security)

## Verification
- [ ] SVID fetch works
- [ ] Auto-rotation functional
- [ ] mTLS connections secure
- [ ] Federation operational
- [ ] Attestation validates
