# Spec 543: Multi-Agent Coordination

## Overview
Coordination protocols for multi-agent task distribution, consensus, leader election, and distributed state management.

## Requirements

### Coordinator Interface
```go
type Coordinator interface {
    // Join cluster
    Join(ctx context.Context, config ClusterConfig) error

    // Leave cluster
    Leave(ctx context.Context) error

    // Get cluster members
    Members(ctx context.Context) ([]*AgentInfo, error)

    // Get leader
    Leader(ctx context.Context) (*AgentInfo, error)

    // Am I leader?
    IsLeader() bool
}
```

### Leader Election
```go
type LeaderElection interface {
    // Campaign for leadership
    Campaign(ctx context.Context) error

    // Resign leadership
    Resign(ctx context.Context) error

    // Watch for leader changes
    Watch(ctx context.Context) (<-chan *LeaderEvent, error)

    // Get current term
    Term() uint64
}

type LeaderEvent struct {
    Type      string    `json:"type"` // elected, resigned, changed
    LeaderID  string    `json:"leaderId"`
    Term      uint64    `json:"term"`
    Timestamp time.Time `json:"timestamp"`
}
```

### Task Distribution
```go
type TaskDistributor interface {
    // Submit task for distribution
    Submit(ctx context.Context, task *DistributedTask) error

    // Claim task for execution
    Claim(ctx context.Context, taskID string) error

    // Complete task
    Complete(ctx context.Context, taskID string, result interface{}) error

    // Fail task
    Fail(ctx context.Context, taskID string, err error) error

    // Get task status
    Status(ctx context.Context, taskID string) (*TaskStatus, error)
}

type DistributedTask struct {
    ID          string                 `json:"id"`
    Type        string                 `json:"type"`
    Priority    int                    `json:"priority"`
    Payload     map[string]interface{} `json:"payload"`
    Constraints TaskConstraints        `json:"constraints"`
    CreatedAt   time.Time             `json:"createdAt"`
    Deadline    *time.Time            `json:"deadline,omitempty"`
}
```

### Task Constraints
```go
type TaskConstraints struct {
    RequiredCapabilities []string `json:"requiredCapabilities"`
    PreferredAgents     []string `json:"preferredAgents,omitempty"`
    ExcludeAgents       []string `json:"excludeAgents,omitempty"`
    MaxRetries          int      `json:"maxRetries"`
    Timeout             time.Duration `json:"timeout"`
    Affinity            string   `json:"affinity,omitempty"`
}
```

### Distributed Locks
```go
type DistributedLock interface {
    // Acquire lock
    Acquire(ctx context.Context, key string, ttl time.Duration) (Lock, error)

    // Try acquire without blocking
    TryAcquire(ctx context.Context, key string, ttl time.Duration) (Lock, bool, error)
}

type Lock interface {
    // Extend lock TTL
    Extend(ctx context.Context, ttl time.Duration) error

    // Release lock
    Release(ctx context.Context) error

    // Check if still held
    IsHeld() bool
}
```

### Consensus Protocol
```go
type ConsensusProtocol interface {
    // Propose value
    Propose(ctx context.Context, key string, value interface{}) error

    // Get agreed value
    Get(ctx context.Context, key string) (interface{}, error)

    // Watch for changes
    Watch(ctx context.Context, prefix string) (<-chan *ConsensusEvent, error)
}
```

### Work Stealing
```go
type WorkStealer interface {
    // Enable work stealing
    Enable(ctx context.Context) error

    // Steal work from busy agents
    Steal(ctx context.Context, targetAgent string) (*DistributedTask, error)

    // Offer work to idle agents
    Offer(ctx context.Context, task *DistributedTask) error
}
```

### Cluster Health
```go
type ClusterHealth struct {
    Healthy      bool              `json:"healthy"`
    MemberCount  int               `json:"memberCount"`
    HealthyCount int               `json:"healthyCount"`
    Leader       string            `json:"leader"`
    Members      []MemberHealth    `json:"members"`
}

type MemberHealth struct {
    ID        string    `json:"id"`
    Healthy   bool      `json:"healthy"`
    LastSeen  time.Time `json:"lastSeen"`
    Load      float64   `json:"load"`
}
```

## Dependencies
- Spec 542: Multi-Agent Communication
- Spec 528: SPIFFE Identity

## Verification
- [ ] Leader election works
- [ ] Task distribution functional
- [ ] Distributed locks work
- [ ] Consensus achieved
- [ ] Failure handling correct
