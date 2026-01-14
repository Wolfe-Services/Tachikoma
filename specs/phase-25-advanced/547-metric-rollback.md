# Spec 547: Metric-Based Rollback

## Overview
Automated rollback system triggered by metric degradation, enabling Tachikoma to automatically revert changes when performance metrics indicate problems.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Rollback Manager Interface
```go
type RollbackManager interface {
    // Create rollback point
    CreateCheckpoint(ctx context.Context, name string) (*Checkpoint, error)

    // Trigger rollback
    Rollback(ctx context.Context, checkpointID string) error

    // Automatic rollback based on metrics
    EnableAutoRollback(ctx context.Context, config AutoRollbackConfig) error

    // Get rollback history
    History(ctx context.Context) ([]*RollbackEvent, error)

    // Get available checkpoints
    Checkpoints(ctx context.Context) ([]*Checkpoint, error)
}
```

### Checkpoint Structure
```go
type Checkpoint struct {
    ID          string                 `json:"id"`
    Name        string                 `json:"name"`
    Timestamp   time.Time             `json:"timestamp"`
    State       map[string]interface{} `json:"state"`
    Configs     map[string]string      `json:"configs"`
    SpecVersion string                 `json:"specVersion"`
    Metrics     MetricsSnapshot        `json:"metrics"`
    CreatedBy   string                 `json:"createdBy"`
    Automatic   bool                   `json:"automatic"`
}
```

### Auto-Rollback Configuration
```go
type AutoRollbackConfig struct {
    Enabled           bool              `json:"enabled"`
    MonitoringWindow  time.Duration     `json:"monitoringWindow"`
    Thresholds        []MetricThreshold `json:"thresholds"`
    CooldownPeriod    time.Duration     `json:"cooldownPeriod"`
    MaxRollbacksPerHour int             `json:"maxRollbacksPerHour"`
    NotifyOnRollback  bool              `json:"notifyOnRollback"`
}

type MetricThreshold struct {
    Metric      string     `json:"metric"`
    Operator    string     `json:"operator"` // >, <, >=, <=, ==
    Value       float64    `json:"value"`
    Duration    time.Duration `json:"duration"` // sustained period
    Severity    string     `json:"severity"` // warning, critical
    AutoRollback bool      `json:"autoRollback"`
}
```

### Metric Monitoring
```go
type MetricMonitor interface {
    // Watch metrics against thresholds
    Watch(ctx context.Context, thresholds []MetricThreshold) (<-chan *ThresholdViolation, error)

    // Get current metric values
    GetMetrics(ctx context.Context, names []string) (map[string]float64, error)

    // Compare with baseline
    CompareWithBaseline(ctx context.Context, baseline *MetricsSnapshot) (*MetricComparison, error)
}

type ThresholdViolation struct {
    Threshold     MetricThreshold `json:"threshold"`
    CurrentValue  float64         `json:"currentValue"`
    Duration      time.Duration   `json:"duration"`
    Timestamp     time.Time       `json:"timestamp"`
    SuggestAction string          `json:"suggestAction"`
}
```

### Rollback Strategies
```go
type RollbackStrategy string

const (
    RollbackStrategyImmediate RollbackStrategy = "immediate"
    RollbackStrategyGraceful  RollbackStrategy = "graceful"
    RollbackStrategyCanary    RollbackStrategy = "canary"
)

type RollbackOptions struct {
    Strategy        RollbackStrategy `json:"strategy"`
    DrainTimeout    time.Duration    `json:"drainTimeout"`
    VerifyAfter     time.Duration    `json:"verifyAfter"`
    NotifyChannels  []string         `json:"notifyChannels"`
}
```

### Rollback Event
```go
type RollbackEvent struct {
    ID            string          `json:"id"`
    CheckpointID  string          `json:"checkpointId"`
    Trigger       RollbackTrigger `json:"trigger"`
    StartTime     time.Time       `json:"startTime"`
    EndTime       *time.Time      `json:"endTime"`
    Status        string          `json:"status"`
    Violations    []ThresholdViolation `json:"violations,omitempty"`
    Error         string          `json:"error,omitempty"`
}

type RollbackTrigger string

const (
    TriggerManual     RollbackTrigger = "manual"
    TriggerAutomatic  RollbackTrigger = "automatic"
    TriggerScheduled  RollbackTrigger = "scheduled"
    TriggerEmergency  RollbackTrigger = "emergency"
)
```

### State Recovery
```go
type StateRecovery interface {
    // Capture current state
    Capture(ctx context.Context) (*StateSnapshot, error)

    // Restore from snapshot
    Restore(ctx context.Context, snapshot *StateSnapshot) error

    // Validate state consistency
    Validate(ctx context.Context) error
}

type StateSnapshot struct {
    ID            string                 `json:"id"`
    Timestamp     time.Time             `json:"timestamp"`
    Configuration map[string]interface{} `json:"configuration"`
    Database      string                `json:"database"` // backup path
    Files         []FileBackup          `json:"files"`
}
```

### Rollback Verification
```go
type RollbackVerifier interface {
    // Verify rollback completed successfully
    VerifyRollback(ctx context.Context, event *RollbackEvent) error

    // Health check after rollback
    HealthCheck(ctx context.Context) (*HealthStatus, error)

    // Verify metrics stabilized
    VerifyMetrics(ctx context.Context, thresholds []MetricThreshold) error
}
```

### Notification Integration
- Slack/Discord alerts
- PagerDuty integration
- Email notifications
- Webhook callbacks

## Dependencies
- Spec 545: Self-Improvement Engine
- Spec 549: Advanced Monitoring

## Verification
- [ ] Checkpoints created correctly
- [ ] Auto-rollback triggers on threshold
- [ ] State restored accurately
- [ ] Metrics verified post-rollback
- [ ] Notifications sent
