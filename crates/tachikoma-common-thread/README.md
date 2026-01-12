# Thread Utilities

Utilities for thread management including named threads, shutdown coordination, and panic handling.

## Features

- **ManagedThread**: Named threads with built-in shutdown coordination
- **ThreadPool**: Simple thread pool with graceful shutdown
- **ShutdownSignal**: Cooperative shutdown mechanism for threads
- **Thread-local storage helpers**: Macros for easier thread-local variables
- **Scoped threads**: Structured concurrency patterns with barrier synchronization
- **Panic handling**: Utilities to catch and convert panics to Results

## Usage

```rust
use tachikoma_common_thread::{ManagedThread, ThreadPool, catch_panic, scoped};

// Named thread with shutdown signal
let thread = ManagedThread::spawn("worker", |signal| {
    while !signal.is_requested() {
        // Do work
    }
});

// Thread pool
let pool = ThreadPool::new(4);
pool.submit(|| println!("Hello from worker thread"));
pool.shutdown();

// Panic handling
let result = catch_panic(|| {
    panic!("Something went wrong");
});

// Scoped threads
scoped::scope(|s| {
    s.spawn(|| println!("Thread 1"));
    s.spawn(|| println!("Thread 2"));
    // All threads complete before scope ends
});
```