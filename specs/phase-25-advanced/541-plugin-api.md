# Spec 541: Plugin API

## Overview
Public API for plugin development, providing interfaces for extending Tachikoma functionality with consistent patterns and host service access.

## Requirements

### Base Plugin Interface
```go
// Plugin is the base interface all plugins must implement
type Plugin interface {
    // Metadata
    Info() *PluginInfo

    // Lifecycle
    Init(ctx context.Context, host HostServices) error
    Start(ctx context.Context) error
    Stop(ctx context.Context) error
}

type PluginInfo struct {
    ID          string
    Name        string
    Version     string
    Description string
    Type        PluginType
}
```

### Host Services
```go
// HostServices provides access to Tachikoma functionality
type HostServices interface {
    // Logging
    Logger() Logger

    // Configuration
    Config() ConfigService

    // Storage
    Storage() StorageService

    // Events
    Events() EventService

    // Secrets
    Secrets() SecretService

    // HTTP client
    HTTP() HTTPService

    // Metrics
    Metrics() MetricsService
}
```

### Task Plugin Interface
```go
type TaskPlugin interface {
    Plugin

    // Execute task
    Execute(ctx context.Context, input TaskInput) (*TaskOutput, error)

    // Validate input
    Validate(ctx context.Context, input TaskInput) error

    // Get schema
    Schema() *TaskSchema
}

type TaskInput struct {
    Params   map[string]interface{} `json:"params"`
    Context  TaskContext            `json:"context"`
}

type TaskOutput struct {
    Result   interface{} `json:"result"`
    Outputs  map[string]interface{} `json:"outputs"`
    Logs     []string    `json:"logs"`
}
```

### Storage Plugin Interface
```go
type StoragePlugin interface {
    Plugin

    // CRUD operations
    Get(ctx context.Context, key string) ([]byte, error)
    Put(ctx context.Context, key string, value []byte) error
    Delete(ctx context.Context, key string) error
    List(ctx context.Context, prefix string) ([]string, error)

    // Transactions
    BeginTx(ctx context.Context) (Transaction, error)
}
```

### Notification Plugin Interface
```go
type NotifyPlugin interface {
    Plugin

    // Send notification
    Send(ctx context.Context, notification Notification) error

    // Get supported channels
    Channels() []string
}

type Notification struct {
    Channel  string                 `json:"channel"`
    Title    string                 `json:"title"`
    Message  string                 `json:"message"`
    Severity string                 `json:"severity"`
    Data     map[string]interface{} `json:"data,omitempty"`
}
```

### Auth Plugin Interface
```go
type AuthPlugin interface {
    Plugin

    // Authenticate user
    Authenticate(ctx context.Context, credentials Credentials) (*Identity, error)

    // Authorize action
    Authorize(ctx context.Context, identity *Identity, action string) (bool, error)

    // Refresh token
    Refresh(ctx context.Context, token string) (*Identity, error)
}
```

### Event Hooks
```go
type EventHook interface {
    // Event types this hook handles
    EventTypes() []string

    // Handle event
    Handle(ctx context.Context, event Event) error

    // Priority (lower = earlier)
    Priority() int
}
```

### Plugin SDK
```go
// SDK helper for plugin development
package sdk

func NewPlugin(info PluginInfo, impl interface{}) Plugin
func WrapTaskPlugin(fn TaskFunc) TaskPlugin
func RegisterHook(hook EventHook)
func GetConfig[T any](host HostServices, key string) (T, error)
```

### Plugin Testing
```go
// Mock host services for testing
type MockHostServices struct {
    LoggerFn   func() Logger
    ConfigFn   func() ConfigService
    // ...
}

// Test harness
func TestPlugin(t *testing.T, plugin Plugin) {
    host := NewMockHostServices()
    err := plugin.Init(context.Background(), host)
    // ...
}
```

## Dependencies
- Spec 539: Plugin Types
- Spec 540: Plugin Loading

## Verification
- [ ] All interfaces defined
- [ ] Host services accessible
- [ ] SDK helpers work
- [ ] Testing utilities functional
- [ ] Documentation complete
