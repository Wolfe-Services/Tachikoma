# Spec 526: Kubernetes Pod Types

## Overview
Define type definitions and data structures for Kubernetes pod management, enabling Tachikoma agents to spawn and manage their own compute pods autonomously.

## Requirements

### Core Pod Types
```go
// PodSpec defines an autonomous Tachikoma pod
type PodSpec struct {
    Name        string            `json:"name"`
    Namespace   string            `json:"namespace"`
    Image       string            `json:"image"`
    Command     []string          `json:"command,omitempty"`
    Args        []string          `json:"args,omitempty"`
    Resources   ResourceSpec      `json:"resources"`
    Volumes     []VolumeMount     `json:"volumes,omitempty"`
    Environment []EnvVar          `json:"environment,omitempty"`
    Labels      map[string]string `json:"labels"`
    Annotations map[string]string `json:"annotations"`
    AgentID     string            `json:"agentId"`
    ParentPod   string            `json:"parentPod,omitempty"`
}

// ResourceSpec defines compute resources
type ResourceSpec struct {
    CPURequest    string `json:"cpuRequest"`
    CPULimit      string `json:"cpuLimit"`
    MemoryRequest string `json:"memoryRequest"`
    MemoryLimit   string `json:"memoryLimit"`
    GPUCount      int    `json:"gpuCount,omitempty"`
    GPUType       string `json:"gpuType,omitempty"`
}
```

### Pod Lifecycle States
```go
type PodState string

const (
    PodStatePending    PodState = "pending"
    PodStateCreating   PodState = "creating"
    PodStateRunning    PodState = "running"
    PodStateCompleted  PodState = "completed"
    PodStateFailed     PodState = "failed"
    PodStateTerminated PodState = "terminated"
)
```

### Pod Hierarchy Types
```go
// PodHierarchy tracks parent-child pod relationships
type PodHierarchy struct {
    RootPod      string   `json:"rootPod"`
    ParentPod    string   `json:"parentPod"`
    ChildPods    []string `json:"childPods"`
    Depth        int      `json:"depth"`
    MaxDepth     int      `json:"maxDepth"`
    CreationTime time.Time `json:"creationTime"`
}
```

### Network Configuration
```go
type PodNetworkConfig struct {
    ServiceAccount   string   `json:"serviceAccount"`
    NetworkPolicy    string   `json:"networkPolicy,omitempty"`
    PodCIDR         string   `json:"podCidr,omitempty"`
    DNSPolicy       string   `json:"dnsPolicy"`
    HostNetwork     bool     `json:"hostNetwork"`
    AllowedPorts    []int    `json:"allowedPorts"`
}
```

### Security Context
```go
type PodSecurityContext struct {
    RunAsUser    int64  `json:"runAsUser"`
    RunAsGroup   int64  `json:"runAsGroup"`
    FSGroup      int64  `json:"fsGroup"`
    RunAsNonRoot bool   `json:"runAsNonRoot"`
    SeccompProfile string `json:"seccompProfile"`
    Capabilities   []string `json:"capabilities,omitempty"`
}
```

### Pod Templates
- Worker pod template (compute-intensive)
- Sidecar pod template (monitoring)
- Init pod template (setup tasks)
- GPU pod template (ML workloads)

## Dependencies
- None (foundational types)

## Verification
- [ ] All types compile
- [ ] JSON serialization works
- [ ] Validation rules defined
- [ ] Kubernetes API compatibility
- [ ] Documentation generated
