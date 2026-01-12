# Spec 539: Plugin Type Definitions

## Overview
Type definitions for the Tachikoma plugin system, enabling extensibility through first-party and third-party plugins with well-defined interfaces.

## Requirements

### Core Plugin Types
```go
// Plugin represents a loadable extension
type Plugin struct {
    ID           string            `json:"id"`
    Name         string            `json:"name"`
    Version      string            `json:"version"`
    Description  string            `json:"description"`
    Author       string            `json:"author"`
    License      string            `json:"license"`
    Homepage     string            `json:"homepage,omitempty"`
    Repository   string            `json:"repository,omitempty"`
    Type         PluginType        `json:"type"`
    Capabilities []Capability      `json:"capabilities"`
    Config       PluginConfig      `json:"config"`
    Dependencies []PluginDep       `json:"dependencies,omitempty"`
    Permissions  []Permission      `json:"permissions"`
}

type PluginType string

const (
    PluginTypeTask       PluginType = "task"      // Task execution
    PluginTypeStorage    PluginType = "storage"   // Storage backend
    PluginTypeAuth       PluginType = "auth"      // Authentication
    PluginTypeNotify     PluginType = "notify"    // Notifications
    PluginTypeTransform  PluginType = "transform" // Data transformation
    PluginTypeMonitor    PluginType = "monitor"   // Monitoring
    PluginTypeIntegration PluginType = "integration" // External service
)
```

### Plugin Capabilities
```go
type Capability string

const (
    CapabilityRead       Capability = "read"
    CapabilityWrite      Capability = "write"
    CapabilityExecute    Capability = "execute"
    CapabilityNetwork    Capability = "network"
    CapabilityFilesystem Capability = "filesystem"
    CapabilityProcess    Capability = "process"
    CapabilitySecret     Capability = "secret"
)
```

### Plugin Configuration
```go
type PluginConfig struct {
    Schema     map[string]ConfigField `json:"schema"`
    Defaults   map[string]interface{} `json:"defaults"`
    Required   []string               `json:"required"`
    Validation []ValidationRule       `json:"validation,omitempty"`
}

type ConfigField struct {
    Type        string      `json:"type"` // string, int, bool, array, object
    Description string      `json:"description"`
    Default     interface{} `json:"default,omitempty"`
    Enum        []string    `json:"enum,omitempty"`
    Minimum     *int        `json:"minimum,omitempty"`
    Maximum     *int        `json:"maximum,omitempty"`
    Secret      bool        `json:"secret"`
}
```

### Plugin Lifecycle
```go
type PluginState string

const (
    PluginStateUnloaded   PluginState = "unloaded"
    PluginStateLoading    PluginState = "loading"
    PluginStateActive     PluginState = "active"
    PluginStateSuspended  PluginState = "suspended"
    PluginStateFailed     PluginState = "failed"
    PluginStateUnloading  PluginState = "unloading"
)
```

### Plugin Permissions
```go
type Permission struct {
    Resource  string   `json:"resource"`
    Actions   []string `json:"actions"`
    Scope     string   `json:"scope,omitempty"`
}
```

### Plugin Manifest
```yaml
# plugin.yaml
id: tachikoma-plugin-slack
name: Slack Notifications
version: 1.0.0
type: notify
description: Send notifications to Slack channels

capabilities:
  - network

permissions:
  - resource: notifications
    actions: [send]

config:
  schema:
    webhook_url:
      type: string
      description: Slack webhook URL
      secret: true
    channel:
      type: string
      default: "#general"
```

### Plugin Registry Entry
```go
type PluginRegistryEntry struct {
    ID            string    `json:"id"`
    Name          string    `json:"name"`
    LatestVersion string    `json:"latestVersion"`
    Versions      []string  `json:"versions"`
    Downloads     int64     `json:"downloads"`
    Rating        float64   `json:"rating"`
    Verified      bool      `json:"verified"`
    UpdatedAt     time.Time `json:"updatedAt"`
}
```

## Dependencies
- None (foundational types)

## Verification
- [ ] Types compile correctly
- [ ] Manifest schema validates
- [ ] Serialization works
- [ ] Permissions model complete
- [ ] Documentation generated
