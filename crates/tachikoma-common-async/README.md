# Tachikoma Common Async

Async runtime utilities for the Tachikoma project.

## Features

- **Runtime Configuration**: Flexible Tokio runtime setup with customizable worker threads, naming, and driver options
- **Graceful Shutdown**: Coordinated shutdown signaling across async tasks
- **Task Utilities**: Named task spawning, timeout handling, and concurrency helpers
- **Runtime Metrics**: Basic metrics collection for monitoring runtime behavior

## Usage

```rust
use tachikoma_common_async::{
    RuntimeConfig, build_runtime, ShutdownHandle,
    spawn_named, with_timeout, join_all
};
use std::time::Duration;

// Configure and build a runtime
let config = RuntimeConfig {
    worker_threads: 4,
    thread_name: "my-app".to_string(),
    enable_io: true,
    enable_time: true,
};

let runtime = build_runtime(config)?;

// Coordinate graceful shutdown
let shutdown = ShutdownHandle::new();
let mut shutdown_rx = shutdown.subscribe();

// Spawn named tasks
let handle = spawn_named("background-worker", async {
    // Task work here
});

// Use timeout for operations
let result = with_timeout(Duration::from_secs(30), async {
    // Some async operation
}).await?;
```

## Testing

Run tests with:

```bash
cargo test -p tachikoma-common-async
```