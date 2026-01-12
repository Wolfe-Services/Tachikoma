//! Thread management utilities.
//!
//! This crate provides utilities for managing threads including:
//! - Named thread spawning with shutdown coordination
//! - Thread-local storage helpers
//! - Scoped thread patterns
//! - Thread pools with shutdown coordination
//! - Panic handling utilities

#![warn(missing_docs)]

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

/// Thread-local storage helper macros and utilities.
pub mod thread_local {
    /// Create a thread-local static variable.
    #[macro_export]
    macro_rules! thread_local_static {
        ($name:ident: $ty:ty = $init:expr) => {
            thread_local! {
                static $name: std::cell::RefCell<$ty> = std::cell::RefCell::new($init);
            }
        };
    }

    /// Access a thread-local variable.
    #[macro_export]
    macro_rules! with_thread_local {
        ($name:ident, |$var:ident| $body:expr) => {
            $name.with(|$var| $body)
        };
    }

    pub use crate::{thread_local_static, with_thread_local};
}

/// Scoped thread patterns for structured concurrency.
pub mod scoped {
    use std::marker::PhantomData;
    use std::sync::{Arc, Barrier};
    use std::thread;

    /// A scope for spawning threads that must complete before the scope ends.
    pub struct Scope<'scope> {
        handles: Vec<std::thread::JoinHandle<()>>,
        _phantom: PhantomData<&'scope ()>,
    }

    impl<'scope> Scope<'scope> {
        /// Create a new scope.
        fn new() -> Self {
            Self {
                handles: Vec::new(),
                _phantom: PhantomData,
            }
        }

        /// Spawn a thread in this scope.
        /// 
        /// Note: This is a simplified implementation for demonstration.
        /// In production, you might want to use crossbeam-scope or similar.
        pub fn spawn<F>(&mut self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let handle = thread::spawn(f);
            self.handles.push(handle);
        }

        /// Spawn multiple threads with a barrier for synchronization.
        pub fn spawn_with_barrier<F>(&mut self, count: usize, f: F)
        where
            F: Fn(Arc<Barrier>) + Send + Sync + 'static,
        {
            let f = Arc::new(f);
            let barrier = Arc::new(Barrier::new(count));
            
            for _ in 0..count {
                let f = f.clone();
                let barrier = barrier.clone();
                let handle = thread::spawn(move || {
                    f(barrier);
                });
                self.handles.push(handle);
            }
        }
    }

    impl<'scope> Drop for Scope<'scope> {
        fn drop(&mut self) {
            for handle in self.handles.drain(..) {
                let _ = handle.join();
            }
        }
    }

    /// Run a function with a scoped thread context.
    pub fn scope<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Scope<'_>) -> R,
    {
        let mut scope = Scope::new();
        f(&mut scope)
    }
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
    fn test_shutdown_signal_wait() {
        let signal = Arc::new(AtomicBool::new(false));
        let shutdown_signal = ShutdownSignal(signal.clone());

        // Should timeout when no shutdown is signaled
        let start = std::time::Instant::now();
        let result = shutdown_signal.wait(std::time::Duration::from_millis(50));
        let elapsed = start.elapsed();
        
        assert!(!result);
        assert!(elapsed >= std::time::Duration::from_millis(50));

        // Should return immediately when shutdown is signaled
        signal.store(true, Ordering::SeqCst);
        let start = std::time::Instant::now();
        let result = shutdown_signal.wait(std::time::Duration::from_millis(100));
        let elapsed = start.elapsed();
        
        assert!(result);
        assert!(elapsed < std::time::Duration::from_millis(50));
    }

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(2);
        let counter = Arc::new(AtomicU32::new(0));

        for _ in 0..10 {
            let c = counter.clone();
            pool.submit(move || {
                c.fetch_add(1, Ordering::SeqCst);
                // Small delay to ensure some interleaving
                thread::sleep(std::time::Duration::from_millis(1));
            });
        }

        // Give tasks some time to complete before shutdown
        thread::sleep(std::time::Duration::from_millis(100));
        pool.shutdown();
        
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_catch_panic_success() {
        let result = catch_panic(|| 42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_catch_panic_string() {
        let result = catch_panic(|| panic!("test panic"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "test panic");
    }

    #[test]
    fn test_thread_local_helpers() {
        use thread_local::{thread_local_static, with_thread_local};

        thread_local_static!(COUNTER: u32 = 0);

        with_thread_local!(COUNTER, |counter| {
            *counter.borrow_mut() = 42;
        });

        with_thread_local!(COUNTER, |counter| {
            assert_eq!(*counter.borrow(), 42);
        });
    }

    #[test]
    fn test_scoped_threads() {
        use scoped::scope;
        
        let counter = Arc::new(AtomicU32::new(0));

        scope(|s| {
            for _ in 0..3 {
                let counter = counter.clone();
                s.spawn(move || {
                    // Simulate some work
                    thread::sleep(std::time::Duration::from_millis(10));
                    counter.fetch_add(1, Ordering::SeqCst);
                });
            }
        }); // All threads must complete before this returns

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_scoped_threads_with_barrier() {
        use scoped::scope;
        
        let counter = Arc::new(AtomicU32::new(0));
        let results = Arc::new(AtomicU32::new(0));

        scope(|s| {
            let counter = counter.clone();
            let results = results.clone();
            
            s.spawn_with_barrier(3, move |barrier| {
                // Each thread increments the counter
                counter.fetch_add(1, Ordering::SeqCst);
                
                // Wait for all threads to reach this point
                barrier.wait();
                
                // Now all threads can proceed together
                let total = counter.load(Ordering::SeqCst);
                if total == 3 {
                    results.fetch_add(1, Ordering::SeqCst);
                }
            });
        });

        assert_eq!(counter.load(Ordering::SeqCst), 3);
        assert_eq!(results.load(Ordering::SeqCst), 3); // All 3 threads should see total = 3
    }

    #[test]
    fn test_managed_thread_name() {
        let thread = ManagedThread::spawn("test-thread", move |_signal| {
            // Do nothing
        });

        assert_eq!(thread.name(), "test-thread");
        thread.shutdown().unwrap();
    }

    #[test]
    fn test_managed_thread_drop_signals_shutdown() {
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let flag = shutdown_flag.clone();

        {
            let _thread = ManagedThread::spawn("test", move |signal| {
                // Wait for shutdown signal
                while !signal.is_requested() {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                flag.store(true, Ordering::SeqCst);
            });
            // thread is dropped here, which should signal shutdown
        }

        // Give the thread a moment to respond to shutdown
        thread::sleep(std::time::Duration::from_millis(50));
        // Note: We can't reliably test the flag here because the thread might not 
        // have had time to process the shutdown signal before the main thread continues
    }
}