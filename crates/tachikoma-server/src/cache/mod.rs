//! Caching layer for the server.
//!
//! Provides both in-memory and Redis cache backends with consistent interface.

pub mod r#trait;
pub mod memory;
pub mod redis;
pub mod helpers;

pub use r#trait::{Cache, CacheError, CacheResult, CacheStats};
pub use memory::MemoryCache;
pub use redis::RedisCache;
pub use helpers::*;