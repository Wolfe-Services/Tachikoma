# Spec 533: eBPF Event Capture

## Overview
eBPF-based event capture system for comprehensive runtime monitoring, security auditing, and performance analysis of Tachikoma agents and spawned processes.

## Requirements

### Capture Manager Interface
```go
type CaptureManager interface {
    // Lifecycle
    Start(ctx context.Context) error
    Stop(ctx context.Context) error

    // Program management
    LoadProgram(ctx context.Context, prog *BPFProgSpec) error
    UnloadProgram(ctx context.Context, name string) error

    // Event streaming
    Subscribe(ctx context.Context, types []EventType) (<-chan Event, error)

    // Filtering
    SetFilter(ctx context.Context, filter *EventFilter) error
}
```

### Event Filter
```go
type EventFilter struct {
    PIDs          []uint32  `json:"pids,omitempty"`
    UIDs          []uint32  `json:"uids,omitempty"`
    CgroupPaths   []string  `json:"cgroupPaths,omitempty"`
    ContainerIDs  []string  `json:"containerIds,omitempty"`
    Comm          []string  `json:"comm,omitempty"`
    ExcludeComm   []string  `json:"excludeComm,omitempty"`
    EventTypes    []uint32  `json:"eventTypes,omitempty"`
}
```

### Probe Attachments
- Kprobes for kernel functions
- Tracepoints for static kernel events
- Uprobes for userspace functions
- Raw tracepoints
- LSM hooks for security

### Supported Probes
```go
var SupportedProbes = []ProbeSpec{
    // Process events
    {Name: "sched_process_exec", Type: "tracepoint"},
    {Name: "sched_process_exit", Type: "tracepoint"},
    {Name: "sched_process_fork", Type: "tracepoint"},

    // Network events
    {Name: "tcp_connect", Type: "kprobe"},
    {Name: "tcp_close", Type: "kprobe"},
    {Name: "udp_sendmsg", Type: "kprobe"},

    // File events
    {Name: "vfs_open", Type: "kprobe"},
    {Name: "vfs_read", Type: "kprobe"},
    {Name: "vfs_write", Type: "kprobe"},

    // Security events
    {Name: "bprm_check_security", Type: "lsm"},
    {Name: "file_permission", Type: "lsm"},
}
```

### Ring Buffer Management
```go
type RingBufferManager interface {
    // Create ring buffer
    Create(ctx context.Context, config RingBufferConfig) error

    // Poll for events
    Poll(ctx context.Context, timeout time.Duration) ([]*Event, error)

    // Read callback
    SetCallback(fn func(*Event)) error

    // Stats
    GetStats() *RingBufferStats
}
```

### Cgroup-Based Filtering
- Container isolation
- Pod-level filtering
- Namespace awareness
- Hierarchical cgroup support

### Event Processing Pipeline
1. eBPF capture in kernel
2. Ring buffer transfer
3. Userspace decoding
4. Event enrichment (container info)
5. Filter application
6. Subscriber dispatch

### Performance Tuning
```go
type PerformanceConfig struct {
    RingBufferSize    uint32  `json:"ringBufferSize"`
    BatchSize         int     `json:"batchSize"`
    PollInterval      time.Duration `json:"pollInterval"`
    MaxEventsPerSec   int     `json:"maxEventsPerSec"`
    SamplingRate      float64 `json:"samplingRate"`
}
```

### Safety Mechanisms
- Kernel version detection
- BTF availability check
- Graceful degradation
- Resource limits
- Program verification

## Dependencies
- Spec 532: eBPF Types

## Verification
- [ ] Programs load correctly
- [ ] Events captured accurately
- [ ] Filters work correctly
- [ ] Performance acceptable
- [ ] No kernel panics
