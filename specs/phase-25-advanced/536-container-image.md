# Spec 536: Container Image Management

## Overview
Container image building, storage, and distribution system for Tachikoma agent deployment, supporting multi-architecture builds and secure image signing.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Image Specification
```go
// ImageSpec defines a container image
type ImageSpec struct {
    Name        string            `json:"name"`
    Tag         string            `json:"tag"`
    Digest      string            `json:"digest"`
    Registry    string            `json:"registry"`
    Repository  string            `json:"repository"`
    Architecture string           `json:"architecture"`
    OS          string            `json:"os"`
    Labels      map[string]string `json:"labels"`
    Created     time.Time         `json:"created"`
    Size        int64             `json:"size"`
}

// ImageManifest for multi-arch support
type ImageManifest struct {
    SchemaVersion int               `json:"schemaVersion"`
    MediaType     string            `json:"mediaType"`
    Manifests     []PlatformManifest `json:"manifests"`
}

type PlatformManifest struct {
    Digest    string   `json:"digest"`
    Size      int64    `json:"size"`
    Platform  Platform `json:"platform"`
}
```

### Image Builder Interface
```go
type ImageBuilder interface {
    // Build image from Dockerfile
    Build(ctx context.Context, opts BuildOptions) (*ImageSpec, error)

    // Build multi-arch image
    BuildMultiArch(ctx context.Context, opts BuildOptions, platforms []Platform) (*ImageManifest, error)

    // Build from source
    BuildFromSource(ctx context.Context, source string, opts BuildOptions) (*ImageSpec, error)
}

type BuildOptions struct {
    Context     string            `json:"context"`
    Dockerfile  string            `json:"dockerfile"`
    Tags        []string          `json:"tags"`
    BuildArgs   map[string]string `json:"buildArgs"`
    Target      string            `json:"target,omitempty"`
    NoCache     bool              `json:"noCache"`
    Pull        bool              `json:"pull"`
    Squash      bool              `json:"squash"`
}
```

### Image Registry Interface
```go
type ImageRegistry interface {
    // Push image to registry
    Push(ctx context.Context, image *ImageSpec, auth *RegistryAuth) error

    // Pull image from registry
    Pull(ctx context.Context, ref string, auth *RegistryAuth) (*ImageSpec, error)

    // List tags for repository
    ListTags(ctx context.Context, repository string) ([]string, error)

    // Delete image
    Delete(ctx context.Context, ref string) error

    // Get manifest
    GetManifest(ctx context.Context, ref string) (*ImageManifest, error)
}
```

### Image Signing (Sigstore/Cosign)
```go
type ImageSigner interface {
    // Sign image
    Sign(ctx context.Context, ref string, key crypto.PrivateKey) (*Signature, error)

    // Verify signature
    Verify(ctx context.Context, ref string, publicKey crypto.PublicKey) (*VerificationResult, error)

    // Attach attestation
    Attest(ctx context.Context, ref string, predicate interface{}) error
}

type Signature struct {
    Digest      string    `json:"digest"`
    Signature   []byte    `json:"signature"`
    Certificate []byte    `json:"certificate,omitempty"`
    Timestamp   time.Time `json:"timestamp"`
}
```

### Base Images
- tachikoma/agent:latest - Full agent
- tachikoma/agent-slim:latest - Minimal agent
- tachikoma/worker:latest - Task worker
- tachikoma/init:latest - Init container

### Vulnerability Scanning
```go
type VulnerabilityScanner interface {
    Scan(ctx context.Context, ref string) (*ScanReport, error)
}

type ScanReport struct {
    Image           string          `json:"image"`
    ScanTime        time.Time       `json:"scanTime"`
    Vulnerabilities []Vulnerability `json:"vulnerabilities"`
    Summary         ScanSummary     `json:"summary"`
}
```

### CI/CD Integration
- GitHub Actions workflow
- GitLab CI pipeline
- Automatic version tagging
- Release automation
- SBOM generation

## Dependencies
- Spec 527: K8s Pod Management

## Verification
- [ ] Multi-arch builds work
- [ ] Image signing functional
- [ ] Vulnerability scanning runs
- [ ] Registry operations work
- [ ] CI/CD pipeline succeeds
