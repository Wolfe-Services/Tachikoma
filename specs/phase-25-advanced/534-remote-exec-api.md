# Spec 534: Remote Execution API

## Overview
API for secure remote command execution on Tachikoma agents, enabling distributed task orchestration with proper authentication, authorization, and auditing.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Execution Request Types
```go
// ExecRequest defines a remote execution request
type ExecRequest struct {
    ID           string            `json:"id"`
    AgentID      string            `json:"agentId"`
    Command      []string          `json:"command"`
    WorkDir      string            `json:"workDir,omitempty"`
    Environment  map[string]string `json:"environment,omitempty"`
    Stdin        []byte            `json:"stdin,omitempty"`
    Timeout      time.Duration     `json:"timeout"`
    User         string            `json:"user,omitempty"`
    Interactive  bool              `json:"interactive"`
    TTY          bool              `json:"tty"`
    RequestedBy  string            `json:"requestedBy"`
    RequestTime  time.Time         `json:"requestTime"`
}

// ExecResponse contains execution results
type ExecResponse struct {
    ID          string        `json:"id"`
    ExitCode    int           `json:"exitCode"`
    Stdout      []byte        `json:"stdout"`
    Stderr      []byte        `json:"stderr"`
    StartTime   time.Time     `json:"startTime"`
    EndTime     time.Time     `json:"endTime"`
    Duration    time.Duration `json:"duration"`
    Error       string        `json:"error,omitempty"`
    Cancelled   bool          `json:"cancelled"`
}
```

### Execution Service Interface
```go
type ExecutionService interface {
    // Execute command
    Execute(ctx context.Context, req *ExecRequest) (*ExecResponse, error)

    // Execute with streaming output
    ExecuteStream(ctx context.Context, req *ExecRequest) (*ExecStream, error)

    // Cancel execution
    Cancel(ctx context.Context, execID string) error

    // Get execution status
    Status(ctx context.Context, execID string) (*ExecStatus, error)

    // List active executions
    List(ctx context.Context, agentID string) ([]*ExecStatus, error)
}
```

### gRPC Service Definition
```protobuf
service RemoteExecution {
    // Unary execution
    rpc Execute(ExecRequest) returns (ExecResponse);

    // Streaming execution
    rpc ExecuteStream(ExecRequest) returns (stream ExecOutput);

    // Bidirectional for interactive
    rpc ExecuteInteractive(stream ExecInput) returns (stream ExecOutput);

    // Control operations
    rpc Cancel(CancelRequest) returns (CancelResponse);
    rpc Status(StatusRequest) returns (StatusResponse);
}
```

### Authorization Policy
```go
type ExecPolicy struct {
    AllowedCommands    []string `json:"allowedCommands"`
    DeniedCommands     []string `json:"deniedCommands"`
    AllowedUsers       []string `json:"allowedUsers"`
    AllowedWorkDirs    []string `json:"allowedWorkDirs"`
    MaxTimeout         time.Duration `json:"maxTimeout"`
    RequireApproval    bool     `json:"requireApproval"`
    AllowInteractive   bool     `json:"allowInteractive"`
    AllowTTY           bool     `json:"allowTty"`
}
```

### Rate Limiting
```go
type RateLimitConfig struct {
    RequestsPerMinute  int `json:"requestsPerMinute"`
    ConcurrentLimit    int `json:"concurrentLimit"`
    QueueSize          int `json:"queueSize"`
    QueueTimeout       time.Duration `json:"queueTimeout"`
}
```

### Audit Logging
```go
type ExecAuditLog struct {
    ID            string    `json:"id"`
    AgentID       string    `json:"agentId"`
    RequestedBy   string    `json:"requestedBy"`
    Command       string    `json:"command"` // sanitized
    ExitCode      int       `json:"exitCode"`
    StartTime     time.Time `json:"startTime"`
    Duration      time.Duration `json:"duration"`
    ClientIP      string    `json:"clientIp"`
    Authorized    bool      `json:"authorized"`
    PolicyApplied string    `json:"policyApplied"`
}
```

### Security Features
- mTLS authentication
- SPIFFE identity verification
- Command allowlist/denylist
- Environment sanitization
- Working directory restrictions
- Resource limits (cgroups)

## Dependencies
- Spec 528: SPIFFE Identity
- Spec 535: Remote Session

## Verification
- [ ] API endpoints work
- [ ] Authorization enforced
- [ ] Streaming functional
- [ ] Audit logs complete
- [ ] Rate limiting works
