//! Test isolation mechanisms to prevent cross-test contamination.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global isolation manager
static ISOLATION_MANAGER: once_cell::sync::Lazy<IsolationManager> = 
    once_cell::sync::Lazy::new(IsolationManager::new);

/// Manages test isolation to prevent cross-test contamination
pub struct IsolationManager {
    active_locks: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
}

impl IsolationManager {
    fn new() -> Self {
        Self {
            active_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Acquire an exclusive lock for a named resource
    pub fn acquire_lock(&self, resource_name: &str) -> ResourceLock {
        let lock_flag = {
            let mut locks = self.active_locks.lock();
            locks.entry(resource_name.to_string())
                .or_insert_with(|| Arc::new(AtomicBool::new(false)))
                .clone()
        };

        // Spin until we can acquire the lock
        while lock_flag.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            std::thread::yield_now();
        }

        ResourceLock {
            resource_name: resource_name.to_string(),
            lock_flag,
        }
    }

    /// Check if a resource is currently locked
    pub fn is_locked(&self, resource_name: &str) -> bool {
        self.active_locks.lock()
            .get(resource_name)
            .map(|flag| flag.load(Ordering::Acquire))
            .unwrap_or(false)
    }
}

/// A resource lock that ensures exclusive access to a named resource
pub struct ResourceLock {
    resource_name: String,
    lock_flag: Arc<AtomicBool>,
}

impl Drop for ResourceLock {
    fn drop(&mut self) {
        self.lock_flag.store(false, Ordering::Release);
        tracing::debug!("Released lock for resource: {}", self.resource_name);
    }
}

/// Acquire a global lock for a resource
pub fn acquire_resource_lock(resource_name: &str) -> ResourceLock {
    tracing::debug!("Acquiring lock for resource: {}", resource_name);
    ISOLATION_MANAGER.acquire_lock(resource_name)
}

/// Check if a resource is currently locked
pub fn is_resource_locked(resource_name: &str) -> bool {
    ISOLATION_MANAGER.is_locked(resource_name)
}

/// Environment variable isolation
pub struct EnvIsolation {
    original_vars: HashMap<String, Option<String>>,
}

impl EnvIsolation {
    /// Create a new environment isolation context
    pub fn new() -> Self {
        Self {
            original_vars: HashMap::new(),
        }
    }

    /// Set an environment variable, saving the original value
    pub fn set_var(&mut self, key: &str, value: &str) {
        self.original_vars.insert(
            key.to_string(),
            std::env::var(key).ok(),
        );
        std::env::set_var(key, value);
    }

    /// Remove an environment variable, saving the original value
    pub fn remove_var(&mut self, key: &str) {
        self.original_vars.insert(
            key.to_string(),
            std::env::var(key).ok(),
        );
        std::env::remove_var(key);
    }
}

impl Drop for EnvIsolation {
    fn drop(&mut self) {
        // Restore original environment variables
        for (key, original_value) in &self.original_vars {
            match original_value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

impl Default for EnvIsolation {
    fn default() -> Self {
        Self::new()
    }
}