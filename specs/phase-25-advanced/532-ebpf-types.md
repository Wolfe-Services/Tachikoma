# Spec 532: eBPF Types and Structures

## Overview
Type definitions for eBPF-based system monitoring, network auditing, and security enforcement in Tachikoma's autonomous infrastructure.

## Requirements

### Event Types
```go
// EventType identifies the eBPF event category
type EventType uint32

const (
    EventTypeProcess   EventType = 1  // Process lifecycle
    EventTypeNetwork   EventType = 2  // Network activity
    EventTypeFile      EventType = 3  // File operations
    EventTypeSyscall   EventType = 4  // System calls
    EventTypeSecurity  EventType = 5  // Security events
)
```

### Process Event Structure
```go
// ProcessEvent captures process lifecycle events
type ProcessEvent struct {
    Timestamp   uint64   `json:"timestamp"`
    EventType   uint32   `json:"eventType"` // exec, exit, fork
    PID         uint32   `json:"pid"`
    PPID        uint32   `json:"ppid"`
    UID         uint32   `json:"uid"`
    GID         uint32   `json:"gid"`
    Comm        [16]byte `json:"comm"`
    Filename    string   `json:"filename"`
    Args        []string `json:"args"`
    ReturnCode  int32    `json:"returnCode"`
    CgroupID    uint64   `json:"cgroupId"`
    ContainerID string   `json:"containerId"`
}
```

### Network Event Structure
```go
// NetworkEvent captures network activity
type NetworkEvent struct {
    Timestamp   uint64   `json:"timestamp"`
    EventType   uint32   `json:"eventType"` // connect, accept, send, recv
    PID         uint32   `json:"pid"`
    Comm        [16]byte `json:"comm"`
    SrcAddr     [16]byte `json:"srcAddr"`
    DstAddr     [16]byte `json:"dstAddr"`
    SrcPort     uint16   `json:"srcPort"`
    DstPort     uint16   `json:"dstPort"`
    Protocol    uint8    `json:"protocol"`
    Family      uint8    `json:"family"` // AF_INET, AF_INET6
    BytesSent   uint64   `json:"bytesSent"`
    BytesRecv   uint64   `json:"bytesRecv"`
}
```

### File Event Structure
```go
// FileEvent captures file operations
type FileEvent struct {
    Timestamp   uint64   `json:"timestamp"`
    EventType   uint32   `json:"eventType"` // open, read, write, unlink
    PID         uint32   `json:"pid"`
    Comm        [16]byte `json:"comm"`
    Filename    string   `json:"filename"`
    Flags       uint32   `json:"flags"`
    Mode        uint32   `json:"mode"`
    ReturnCode  int32    `json:"returnCode"`
    BytesRead   uint64   `json:"bytesRead"`
    BytesWritten uint64  `json:"bytesWritten"`
}
```

### Security Event Structure
```go
// SecurityEvent captures security-relevant events
type SecurityEvent struct {
    Timestamp   uint64   `json:"timestamp"`
    EventType   uint32   `json:"eventType"`
    Severity    uint8    `json:"severity"`
    PID         uint32   `json:"pid"`
    UID         uint32   `json:"uid"`
    Action      string   `json:"action"` // blocked, allowed, alert
    Reason      string   `json:"reason"`
    Details     string   `json:"details"`
}
```

### Map Types
```go
// BPFMapSpec defines a BPF map
type BPFMapSpec struct {
    Name       string  `json:"name"`
    Type       uint32  `json:"type"` // hash, array, ringbuf, etc.
    KeySize    uint32  `json:"keySize"`
    ValueSize  uint32  `json:"valueSize"`
    MaxEntries uint32  `json:"maxEntries"`
    Flags      uint32  `json:"flags"`
}
```

### Program Types
```go
// BPFProgSpec defines a BPF program
type BPFProgSpec struct {
    Name       string `json:"name"`
    Type       uint32 `json:"type"` // kprobe, tracepoint, etc.
    AttachType uint32 `json:"attachType"`
    License    string `json:"license"`
    BTF        bool   `json:"btf"`
}
```

### Ring Buffer Configuration
```go
type RingBufferConfig struct {
    SizeBytes    uint32        `json:"sizeBytes"`
    WakeupBytes  uint32        `json:"wakeupBytes"`
    PollTimeout  time.Duration `json:"pollTimeout"`
}
```

## Dependencies
- None (foundational types)

## Verification
- [ ] Types compile correctly
- [ ] Serialization works
- [ ] Kernel compatibility verified
- [ ] Documentation complete
- [ ] Test coverage adequate
