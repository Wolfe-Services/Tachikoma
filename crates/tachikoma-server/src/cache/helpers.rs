//! Cache helper functions and patterns.

use super::r#trait::{Cache, CacheResult};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::time::Duration;

/// Cache-aside pattern helper.
/// 
/// Tries to get a value from cache first. If not found, fetches from source,
/// stores in cache, and returns the value.
pub async fn cache_aside<T, F, Fut>(
    cache: &dyn Cache,
    key: &str,
    ttl: Duration,
    fetch: F,
) -> CacheResult<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
    F: FnOnce() -> Fut,
    Fut: Future<Output = CacheResult<T>>,
{
    // Try cache first
    if let Some(value) = cache.get::<T>(key).await? {
        return Ok(value);
    }

    // Fetch from source
    let value = fetch().await?;

    // Store in cache
    cache.set(key, &value, ttl).await?;

    Ok(value)
}

/// Cache key builder for consistent key generation.
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    /// Create a new key builder with the given prefix.
    pub fn new(prefix: &str) -> Self {
        Self {
            parts: vec![prefix.to_string()],
        }
    }

    /// Add a part to the key.
    pub fn add(mut self, part: impl ToString) -> Self {
        self.parts.push(part.to_string());
        self
    }

    /// Add an optional part to the key.
    pub fn add_opt(mut self, part: Option<impl ToString>) -> Self {
        if let Some(p) = part {
            self.parts.push(p.to_string());
        }
        self
    }

    /// Build the final key by joining parts with colons.
    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

/// Common cache key prefixes.
pub mod keys {
    use super::CacheKeyBuilder;
    use uuid::Uuid;

    /// Cache key for a specific mission.
    pub fn mission(id: Uuid) -> String {
        CacheKeyBuilder::new("mission").add(id).build()
    }

    /// Cache key for mission list with pagination.
    pub fn mission_list(page: u32, page_size: u32) -> String {
        CacheKeyBuilder::new("missions")
            .add("list")
            .add(page)
            .add(page_size)
            .build()
    }

    /// Cache key for a spec within a mission.
    pub fn spec(mission_id: Uuid, spec_id: &str) -> String {
        CacheKeyBuilder::new("spec")
            .add(mission_id)
            .add(spec_id)
            .build()
    }

    /// Cache key for a user.
    pub fn user(id: Uuid) -> String {
        CacheKeyBuilder::new("user").add(id).build()
    }

    /// Cache key for a user session.
    pub fn session(token: &str) -> String {
        CacheKeyBuilder::new("session").add(token).build()
    }
}

/// Invalidation patterns for bulk cache clearing.
pub mod invalidate {
    use uuid::Uuid;

    /// Pattern to invalidate all cache entries for a specific mission.
    pub fn mission(id: Uuid) -> String {
        format!("mission:{}*", id)
    }

    /// Pattern to invalidate all mission-related cache entries.
    pub fn all_missions() -> String {
        "missions:*".to_string()
    }

    /// Pattern to invalidate all cache entries for a specific user.
    pub fn user(id: Uuid) -> String {
        format!("user:{}*", id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_builder() {
        let key = CacheKeyBuilder::new("mission")
            .add("123")
            .add("spec")
            .add("456")
            .build();
        
        assert_eq!(key, "mission:123:spec:456");
    }

    #[test]
    fn test_cache_key_builder_with_optional() {
        let key = CacheKeyBuilder::new("missions")
            .add("list")
            .add_opt(Some(1))
            .add_opt(None::<String>)
            .add(10)
            .build();
        
        assert_eq!(key, "missions:list:1:10");
    }

    #[test]
    fn test_predefined_keys() {
        use uuid::Uuid;
        
        let mission_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        assert_eq!(keys::mission(mission_id), format!("mission:{}", mission_id));
        assert_eq!(keys::mission_list(1, 10), "missions:list:1:10");
        assert_eq!(keys::spec(mission_id, "spec-123"), format!("spec:{}:spec-123", mission_id));
        assert_eq!(keys::user(user_id), format!("user:{}", user_id));
        assert_eq!(keys::session("token-abc"), "session:token-abc");
    }

    #[test]
    fn test_invalidation_patterns() {
        use uuid::Uuid;
        
        let mission_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        assert_eq!(invalidate::mission(mission_id), format!("mission:{}*", mission_id));
        assert_eq!(invalidate::all_missions(), "missions:*");
        assert_eq!(invalidate::user(user_id), format!("user:{}*", user_id));
    }
}