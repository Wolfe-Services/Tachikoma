# Spec 540: Plugin Loading System

## Overview
Dynamic plugin loading system supporting Go plugins, WebAssembly modules, and external process plugins with sandboxing and lifecycle management.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Plugin Loader Interface
```go
type PluginLoader interface {
    // Load plugin from path
    Load(ctx context.Context, path string) (*LoadedPlugin, error)

    // Load from registry
    LoadFromRegistry(ctx context.Context, id, version string) (*LoadedPlugin, error)

    // Unload plugin
    Unload(ctx context.Context, pluginID string) error

    // Reload plugin
    Reload(ctx context.Context, pluginID string) error

    // List loaded plugins
    List(ctx context.Context) ([]*LoadedPlugin, error)
}
```

### Loaded Plugin
```go
type LoadedPlugin struct {
    Plugin      *Plugin     `json:"plugin"`
    State       PluginState `json:"state"`
    LoadedAt    time.Time   `json:"loadedAt"`
    Instance    interface{} `json:"-"`
    Handle      PluginHandle `json:"-"`
    Error       string      `json:"error,omitempty"`
}

type PluginHandle interface {
    Call(method string, args ...interface{}) (interface{}, error)
    Close() error
}
```

### Go Plugin Loading
```go
type GoPluginLoader struct {
    SearchPaths []string
    SymbolMap   map[string]string
}

func (l *GoPluginLoader) Load(ctx context.Context, path string) (*LoadedPlugin, error) {
    plug, err := plugin.Open(path)
    if err != nil {
        return nil, err
    }

    // Look up required symbols
    initSym, err := plug.Lookup("Init")
    // ...
}
```

### WebAssembly Plugin Loading
```go
type WasmPluginLoader struct {
    Runtime    wazero.Runtime
    ModuleConfig wazero.ModuleConfig
    MemoryLimit uint64
}

type WasmPluginHandle struct {
    Module  api.Module
    Exports map[string]api.Function
}
```

### External Process Plugin
```go
type ExternalPluginLoader struct {
    PluginDir     string
    Protocol      string // jsonrpc, grpc
    Handshake     HandshakeConfig
}

type ExternalPluginHandle struct {
    Process   *os.Process
    Client    *rpc.Client
    Protocol  string
}
```

### Plugin Discovery
```go
type PluginDiscovery interface {
    // Scan directories for plugins
    Scan(ctx context.Context, paths []string) ([]*DiscoveredPlugin, error)

    // Watch for new plugins
    Watch(ctx context.Context, paths []string) (<-chan *DiscoveredPlugin, error)
}

type DiscoveredPlugin struct {
    Path        string     `json:"path"`
    Type        LoaderType `json:"type"` // go, wasm, external
    Manifest    *Plugin    `json:"manifest"`
    Checksum    string     `json:"checksum"`
}
```

### Sandbox Configuration
```go
type SandboxConfig struct {
    Enabled         bool     `json:"enabled"`
    AllowedPaths    []string `json:"allowedPaths"`
    DeniedPaths     []string `json:"deniedPaths"`
    NetworkAllowed  bool     `json:"networkAllowed"`
    AllowedHosts    []string `json:"allowedHosts"`
    MemoryLimit     uint64   `json:"memoryLimit"`
    CPULimit        float64  `json:"cpuLimit"`
    Timeout         time.Duration `json:"timeout"`
}
```

### Plugin Initialization
```go
type PluginInitializer interface {
    // Initialize plugin with config
    Init(ctx context.Context, config map[string]interface{}) error

    // Health check
    HealthCheck(ctx context.Context) error

    // Cleanup
    Shutdown(ctx context.Context) error
}
```

### Hot Reloading
- File watcher for plugin changes
- Graceful plugin replacement
- State migration between versions
- Connection draining
- Rollback on failure

## Dependencies
- Spec 539: Plugin Types

## Verification
- [ ] Go plugins load correctly
- [ ] WASM plugins execute
- [ ] External plugins communicate
- [ ] Sandbox limits enforced
- [ ] Hot reload works
