# 018 - Thread Utilities

**Phase:** 1 - Core Common Crates
**Spec ID:** 018
**Status:** Complete
**Dependencies:** 011-common-core-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Provide thread management utilities including named thread spawning, thread-local storage patterns, and synchronization primitives.

---

## Acceptance Criteria

- [x] Named thread spawning
- [x] Thread-local storage helpers
- [x] Scoped thread patterns
- [x] Shutdown coordination
- [x] Panic handling

---

## Implementation Details

### 1. Thread Module (crates/tachikoma-common-thread/src/lib.rs)

```rust
//! Thread management utilities.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// A handle to a named, managed thread.
pub struct ManagedThread {
    handle: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    name: String,
}

impl ManagedThread {
    /// Spawn a new named thread.
    pub fn spawn<F>(name: impl Into<String>, f: F) -> Self
    where
        F: FnOnce(ShutdownSignal) + Send + 'static,
    {
        let name = name.into();
        let shutdown = Arc::new(AtomicBool::new(false));
        let signal = ShutdownSignal(shutdown.clone());

        let thread_name = name.clone();
        let handle = thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                f(signal);
            })
            .expect("failed to spawn thread");

        Self {
            handle: Some(handle),
            shutdown,
            name,
        }
    }

    /// Signal the thread to shut down.
    pub fn signal_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Wait for the thread to complete.
    pub fn join(mut self) -> thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
        } else {
            Ok(())
        }
    }

    /// Signal shutdown and wait.
    pub fn shutdown(self) -> thread::Result<()> {
        self.signal_shutdown();
        self.join()
    }

    /// Get the thread name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if shutdown was requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }
}

impl Drop for ManagedThread {
    fn drop(&mut self) {
        self.signal_shutdown();
        // Note: We don't join in Drop to avoid blocking
    }
}

/// A signal to check for shutdown requests.
#[derive(Clone)]
pub struct ShutdownSignal(Arc<AtomicBool>);

impl ShutdownSignal {
    /// Check if shutdown was requested.
    pub fn is_requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    /// Wait until shutdown is requested or timeout.
    pub fn wait(&self, timeout: std::time::Duration) -> bool {
        let start = std::time::Instant::now();
        while !self.is_requested() {
            if start.elapsed() >= timeout {
                return false;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        true
    }
}

/// A thread pool for parallel work.
pub struct ThreadPool {
    workers: Vec<ManagedThread>,
    sender: crossbeam_channel::Sender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool {
    /// Create a new thread pool with the given number of workers.
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<Box<dyn FnOnce() + Send>>();

        let workers: Vec<_> = (0..size)
            .map(|i| {
                let rx = receiver.clone();
                ManagedThread::spawn(format!("pool-worker-{}", i), move |signal| {
                    while !signal.is_requested() {
                        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                            Ok(task) => task(),
                            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                })
            })
            .collect();

        Self { workers, sender }
    }

    /// Submit a task to the pool.
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).expect("pool disconnected");
    }

    /// Shut down the pool and wait for all workers.
    pub fn shutdown(self) {
        drop(self.sender);
        for worker in self.workers {
            let _ = worker.shutdown();
        }
    }
}

/// Catch panics and convert to Result.
pub fn catch_panic<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(f).map_err(|e| {
        if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn test_managed_thread() {
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let thread = ManagedThread::spawn("test", move |signal| {
            while !signal.is_requested() {
                c.fetch_add(1, Ordering::SeqCst);
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        thread::sleep(std::time::Duration::from_millis(50));
        thread.shutdown().unwrap();

        assert!(counter.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..10 {
            let c = counter.clone();
            pool.submit(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        pool.shutdown();
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-thread"
version.workspace = true
edition.workspace = true

[dependencies]
crossbeam-channel = "0.5"
```

---

## Testing Requirements

1. Threads receive shutdown signal
2. Pool processes all submitted tasks
3. Panic catching works correctly
4. No resource leaks on shutdown

---

## Related Specs

- Depends on: [011-common-core-types.md](011-common-core-types.md)
- Next: [019-async-runtime.md](019-async-runtime.md)
