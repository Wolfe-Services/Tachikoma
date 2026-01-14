# Spec 538: Self-Update Mechanism

## Overview
Autonomous self-update capability for Tachikoma agents, enabling automatic version upgrades with rollback support, integrity verification, and zero-downtime updates.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Update Manager Interface
```go
type UpdateManager interface {
    // Check for updates
    CheckUpdate(ctx context.Context) (*UpdateInfo, error)

    // Download update
    Download(ctx context.Context, version string) (*UpdatePackage, error)

    // Apply update
    Apply(ctx context.Context, pkg *UpdatePackage) error

    // Rollback to previous version
    Rollback(ctx context.Context) error

    // Get current version
    CurrentVersion() string

    // Get update history
    History(ctx context.Context) ([]*UpdateRecord, error)
}
```

### Update Information
```go
type UpdateInfo struct {
    CurrentVersion   string    `json:"currentVersion"`
    LatestVersion    string    `json:"latestVersion"`
    UpdateAvailable  bool      `json:"updateAvailable"`
    ReleaseNotes     string    `json:"releaseNotes"`
    ReleaseDate      time.Time `json:"releaseDate"`
    Mandatory        bool      `json:"mandatory"`
    SecurityFix      bool      `json:"securityFix"`
    DownloadURL      string    `json:"downloadUrl"`
    Checksum         string    `json:"checksum"`
    Signature        string    `json:"signature"`
}
```

### Update Package
```go
type UpdatePackage struct {
    Version    string `json:"version"`
    Platform   string `json:"platform"`
    Arch       string `json:"arch"`
    Binary     []byte `json:"-"`
    Checksum   string `json:"checksum"`
    Signature  string `json:"signature"`
    Size       int64  `json:"size"`
}
```

### Update Policy
```go
type UpdatePolicy struct {
    Enabled           bool          `json:"enabled"`
    AutoUpdate        bool          `json:"autoUpdate"`
    Channel           string        `json:"channel"` // stable, beta, nightly
    CheckInterval     time.Duration `json:"checkInterval"`
    MaintenanceWindow *TimeWindow   `json:"maintenanceWindow,omitempty"`
    RequireApproval   bool          `json:"requireApproval"`
    AllowDowngrade    bool          `json:"allowDowngrade"`
}

type TimeWindow struct {
    DayOfWeek  []int  `json:"dayOfWeek"` // 0-6
    StartHour  int    `json:"startHour"`
    EndHour    int    `json:"endHour"`
    Timezone   string `json:"timezone"`
}
```

### Integrity Verification
```go
type IntegrityVerifier interface {
    // Verify checksum
    VerifyChecksum(data []byte, expected string) error

    // Verify signature
    VerifySignature(data []byte, signature string, publicKey []byte) error

    // Verify certificate chain
    VerifyCertChain(cert []byte) error
}
```

### Update Process
1. Check for available updates
2. Verify update authenticity
3. Download update package
4. Verify integrity (checksum + signature)
5. Backup current binary
6. Apply update (atomic swap)
7. Verify new binary works
8. Clean up or rollback

### Zero-Downtime Update
```go
type GracefulUpdater interface {
    // Prepare update (download, verify)
    Prepare(ctx context.Context) error

    // Signal workers to drain
    DrainConnections(ctx context.Context, timeout time.Duration) error

    // Execute update
    Execute(ctx context.Context) error

    // Verify health post-update
    HealthCheck(ctx context.Context) error
}
```

### Rollback Management
```go
type RollbackManager interface {
    // Create restore point
    CreateRestorePoint(ctx context.Context, version string) error

    // List restore points
    ListRestorePoints(ctx context.Context) ([]*RestorePoint, error)

    // Rollback to restore point
    Rollback(ctx context.Context, restoreID string) error

    // Cleanup old restore points
    Cleanup(ctx context.Context, keepCount int) error
}
```

### Update Channels
- stable: Production-ready releases
- beta: Pre-release testing
- nightly: Daily development builds
- security: Emergency security patches

## Dependencies
- Spec 536: Container Image (for container updates)

## Verification
- [ ] Update check works
- [ ] Download and verify work
- [ ] Atomic update applies
- [ ] Rollback functional
- [ ] Zero-downtime achieved
