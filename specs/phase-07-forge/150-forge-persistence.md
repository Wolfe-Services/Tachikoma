# 150 - Forge Session Persistence

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 150
**Status:** Planned
**Dependencies:** 136-forge-session-types, 137-forge-config
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement persistence for Forge sessions, allowing sessions to be saved to disk, auto-saved periodically, and loaded for review or resumption.

---

## Acceptance Criteria

- [ ] Session serialization to JSON/YAML
- [ ] Auto-save at configurable intervals
- [ ] Manual save on demand
- [ ] Session listing and discovery
- [ ] Session metadata storage
- [ ] Atomic write operations
- [ ] Compression for large sessions

---

## Implementation Details

### 1. Session Store (src/persistence/store.rs)

```rust
//! Session persistence storage.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;

use crate::{
    ForgeConfig, ForgeError, ForgeResult, ForgeSession, ForgeSessionId,
    ForgeSessionStatus, Timestamp,
};

/// Persistent storage for Forge sessions.
pub struct SessionStore {
    /// Base directory for session storage.
    base_dir: PathBuf,
    /// Session metadata cache.
    metadata_cache: Arc<RwLock<HashMap<ForgeSessionId, SessionMetadata>>>,
    /// Configuration.
    config: ForgeConfig,
}

/// Metadata about a stored session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMetadata {
    /// Session ID.
    pub id: ForgeSessionId,
    /// Session name.
    pub name: String,
    /// Session status.
    pub status: ForgeSessionStatus,
    /// Topic title.
    pub topic_title: String,
    /// Created timestamp.
    pub created_at: Timestamp,
    /// Last modified timestamp.
    pub updated_at: Timestamp,
    /// Number of rounds completed.
    pub round_count: usize,
    /// Total cost so far.
    pub total_cost_usd: f64,
    /// File size in bytes.
    pub file_size: u64,
    /// Is compressed.
    pub compressed: bool,
}

impl SessionStore {
    /// Create a new session store.
    pub async fn new(config: ForgeConfig) -> ForgeResult<Self> {
        let base_dir = config.persistence.session_dir.clone();

        // Ensure directory exists
        fs::create_dir_all(&base_dir).await
            .map_err(|e| ForgeError::Io(format!("Failed to create session dir: {}", e)))?;

        let store = Self {
            base_dir,
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // Load metadata cache
        store.refresh_metadata_cache().await?;

        Ok(store)
    }

    /// Save a session.
    pub async fn save(&self, session: &ForgeSession) -> ForgeResult<PathBuf> {
        let session_path = self.session_path(&session.id);

        // Serialize session
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| ForgeError::Serialization(format!("Failed to serialize session: {}", e)))?;

        // Determine if we should compress
        let (data, compressed) = if json.len() > 100_000 {
            // Compress large sessions
            let compressed = compress_data(json.as_bytes())?;
            (compressed, true)
        } else {
            (json.into_bytes(), false)
        };

        // Atomic write using temp file
        let temp_path = session_path.with_extension("tmp");
        fs::write(&temp_path, &data).await
            .map_err(|e| ForgeError::Io(format!("Failed to write session: {}", e)))?;

        // Rename to final path
        fs::rename(&temp_path, &session_path).await
            .map_err(|e| ForgeError::Io(format!("Failed to finalize session save: {}", e)))?;

        // Update metadata cache
        let metadata = SessionMetadata {
            id: session.id.clone(),
            name: session.name.clone(),
            status: session.status,
            topic_title: session.topic.title.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            round_count: session.rounds.len(),
            total_cost_usd: session.total_cost_usd,
            file_size: data.len() as u64,
            compressed,
        };

        self.metadata_cache.write().await.insert(session.id.clone(), metadata);

        // Save metadata index
        self.save_metadata_index().await?;

        Ok(session_path)
    }

    /// Load a session by ID.
    pub async fn load(&self, session_id: &ForgeSessionId) -> ForgeResult<ForgeSession> {
        let session_path = self.session_path(session_id);

        if !session_path.exists() {
            return Err(ForgeError::NotFound(format!("Session {} not found", session_id)));
        }

        let data = fs::read(&session_path).await
            .map_err(|e| ForgeError::Io(format!("Failed to read session: {}", e)))?;

        // Check if compressed
        let json = if is_compressed(&data) {
            decompress_data(&data)?
        } else {
            String::from_utf8(data)
                .map_err(|e| ForgeError::Serialization(format!("Invalid UTF-8: {}", e)))?
        };

        let session: ForgeSession = serde_json::from_str(&json)
            .map_err(|e| ForgeError::Serialization(format!("Failed to deserialize session: {}", e)))?;

        Ok(session)
    }

    /// Delete a session.
    pub async fn delete(&self, session_id: &ForgeSessionId) -> ForgeResult<()> {
        let session_path = self.session_path(session_id);

        if session_path.exists() {
            fs::remove_file(&session_path).await
                .map_err(|e| ForgeError::Io(format!("Failed to delete session: {}", e)))?;
        }

        self.metadata_cache.write().await.remove(session_id);
        self.save_metadata_index().await?;

        Ok(())
    }

    /// List all sessions.
    pub async fn list(&self) -> ForgeResult<Vec<SessionMetadata>> {
        let cache = self.metadata_cache.read().await;
        let mut sessions: Vec<_> = cache.values().cloned().collect();

        // Sort by updated_at descending
        sessions.sort_by(|a, b| b.updated_at.as_datetime().cmp(&a.updated_at.as_datetime()));

        Ok(sessions)
    }

    /// List sessions by status.
    pub async fn list_by_status(&self, status: ForgeSessionStatus) -> ForgeResult<Vec<SessionMetadata>> {
        let all = self.list().await?;
        Ok(all.into_iter().filter(|s| s.status == status).collect())
    }

    /// Check if a session exists.
    pub async fn exists(&self, session_id: &ForgeSessionId) -> bool {
        self.session_path(session_id).exists()
    }

    /// Get session metadata.
    pub async fn get_metadata(&self, session_id: &ForgeSessionId) -> Option<SessionMetadata> {
        self.metadata_cache.read().await.get(session_id).cloned()
    }

    /// Cleanup old sessions.
    pub async fn cleanup(&self) -> ForgeResult<usize> {
        let sessions = self.list().await?;

        if sessions.len() <= self.config.persistence.max_sessions {
            return Ok(0);
        }

        // Keep only max_sessions, prioritizing in-progress over completed
        let mut to_keep: Vec<_> = sessions.iter()
            .filter(|s| s.status == ForgeSessionStatus::InProgress || s.status == ForgeSessionStatus::Paused)
            .collect();

        let remaining_slots = self.config.persistence.max_sessions.saturating_sub(to_keep.len());

        to_keep.extend(
            sessions.iter()
                .filter(|s| s.status != ForgeSessionStatus::InProgress && s.status != ForgeSessionStatus::Paused)
                .take(remaining_slots)
        );

        let keep_ids: std::collections::HashSet<_> = to_keep.iter().map(|s| &s.id).collect();

        let mut deleted = 0;
        for session in &sessions {
            if !keep_ids.contains(&session.id) {
                self.delete(&session.id).await?;
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// Get the path for a session file.
    fn session_path(&self, session_id: &ForgeSessionId) -> PathBuf {
        self.base_dir.join(format!("{}.json", session_id))
    }

    /// Refresh metadata cache from disk.
    async fn refresh_metadata_cache(&self) -> ForgeResult<()> {
        // Try to load metadata index
        let index_path = self.base_dir.join("index.json");

        if index_path.exists() {
            let data = fs::read_to_string(&index_path).await
                .map_err(|e| ForgeError::Io(format!("Failed to read index: {}", e)))?;

            let metadata: HashMap<String, SessionMetadata> = serde_json::from_str(&data)
                .unwrap_or_default();

            let mut cache = self.metadata_cache.write().await;
            for (id_str, meta) in metadata {
                if let Ok(id) = id_str.parse() {
                    cache.insert(id, meta);
                }
            }
        }

        // Also scan directory for any sessions not in index
        let mut entries = fs::read_dir(&self.base_dir).await
            .map_err(|e| ForgeError::Io(format!("Failed to read session dir: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ForgeError::Io(format!("Failed to read dir entry: {}", e)))?
        {
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false)
                && path.file_name().map(|n| n != "index.json").unwrap_or(false)
            {
                // Extract session ID from filename
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(session_id) = stem.parse::<ForgeSessionId>() {
                        let cache = self.metadata_cache.read().await;
                        if !cache.contains_key(&session_id) {
                            drop(cache);
                            // Load session to get metadata
                            if let Ok(session) = self.load(&session_id).await {
                                let file_size = fs::metadata(&path).await
                                    .map(|m| m.len())
                                    .unwrap_or(0);

                                let metadata = SessionMetadata {
                                    id: session.id.clone(),
                                    name: session.name.clone(),
                                    status: session.status,
                                    topic_title: session.topic.title.clone(),
                                    created_at: session.created_at,
                                    updated_at: session.updated_at,
                                    round_count: session.rounds.len(),
                                    total_cost_usd: session.total_cost_usd,
                                    file_size,
                                    compressed: false,
                                };

                                self.metadata_cache.write().await.insert(session_id, metadata);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Save metadata index.
    async fn save_metadata_index(&self) -> ForgeResult<()> {
        let cache = self.metadata_cache.read().await;

        let index: HashMap<String, &SessionMetadata> = cache.iter()
            .map(|(id, meta)| (id.to_string(), meta))
            .collect();

        let json = serde_json::to_string_pretty(&index)
            .map_err(|e| ForgeError::Serialization(format!("Failed to serialize index: {}", e)))?;

        let index_path = self.base_dir.join("index.json");
        fs::write(&index_path, json).await
            .map_err(|e| ForgeError::Io(format!("Failed to write index: {}", e)))?;

        Ok(())
    }
}

/// Compress data using zstd.
fn compress_data(data: &[u8]) -> ForgeResult<Vec<u8>> {
    // Simple compression using zstd
    zstd::encode_all(data, 3)
        .map_err(|e| ForgeError::Io(format!("Compression failed: {}", e)))
}

/// Decompress data.
fn decompress_data(data: &[u8]) -> ForgeResult<String> {
    let decompressed = zstd::decode_all(data)
        .map_err(|e| ForgeError::Io(format!("Decompression failed: {}", e)))?;

    String::from_utf8(decompressed)
        .map_err(|e| ForgeError::Serialization(format!("Invalid UTF-8 after decompression: {}", e)))
}

/// Check if data is compressed (zstd magic number).
fn is_compressed(data: &[u8]) -> bool {
    data.len() >= 4 && data[0..4] == [0x28, 0xB5, 0x2F, 0xFD]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ForgeConfig::default();
        config.persistence.session_dir = temp_dir.path().to_path_buf();

        let store = SessionStore::new(config).await.unwrap();

        let session = ForgeSession::new(
            "Test Session",
            crate::BrainstormTopic::new("Test", "Description")
        );

        store.save(&session).await.unwrap();

        let loaded = store.load(&session.id).await.unwrap();
        assert_eq!(loaded.name, "Test Session");
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = ForgeConfig::default();
        config.persistence.session_dir = temp_dir.path().to_path_buf();

        let store = SessionStore::new(config).await.unwrap();

        // Save multiple sessions
        for i in 0..3 {
            let session = ForgeSession::new(
                format!("Session {}", i),
                crate::BrainstormTopic::new("Test", "Description")
            );
            store.save(&session).await.unwrap();
        }

        let sessions = store.list().await.unwrap();
        assert_eq!(sessions.len(), 3);
    }
}
```

### 2. Auto-Save Manager (src/persistence/autosave.rs)

```rust
//! Auto-save functionality for Forge sessions.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

use crate::{ForgeConfig, ForgeResult, ForgeSession, SessionStore};

/// Manager for auto-saving sessions.
pub struct AutoSaveManager {
    store: Arc<SessionStore>,
    config: ForgeConfig,
    /// Current session being tracked.
    session: Arc<RwLock<Option<ForgeSession>>>,
    /// Shutdown signal.
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl AutoSaveManager {
    /// Create a new auto-save manager.
    pub fn new(store: Arc<SessionStore>, config: ForgeConfig) -> Self {
        Self {
            store,
            config,
            session: Arc::new(RwLock::new(None)),
            shutdown_tx: None,
        }
    }

    /// Start auto-saving a session.
    pub fn start(&mut self, session: ForgeSession) -> mpsc::Sender<ForgeSession> {
        let (update_tx, mut update_rx) = mpsc::channel::<ForgeSession>(10);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        self.shutdown_tx = Some(shutdown_tx);

        let session_arc = self.session.clone();
        let store = self.store.clone();
        let interval_secs = self.config.persistence.auto_save_interval_secs;

        // Set initial session
        {
            let mut guard = session_arc.blocking_write();
            *guard = Some(session);
        }

        // Spawn auto-save task
        tokio::spawn(async move {
            let mut save_interval = interval(Duration::from_secs(interval_secs));

            loop {
                tokio::select! {
                    // Handle session updates
                    Some(updated_session) = update_rx.recv() => {
                        let mut guard = session_arc.write().await;
                        *guard = Some(updated_session);
                    }

                    // Periodic save
                    _ = save_interval.tick() => {
                        let guard = session_arc.read().await;
                        if let Some(ref session) = *guard {
                            if let Err(e) = store.save(session).await {
                                tracing::error!("Auto-save failed: {}", e);
                            } else {
                                tracing::debug!("Auto-saved session {}", session.id);
                            }
                        }
                    }

                    // Shutdown signal
                    _ = shutdown_rx.recv() => {
                        // Final save before shutdown
                        let guard = session_arc.read().await;
                        if let Some(ref session) = *guard {
                            if let Err(e) = store.save(session).await {
                                tracing::error!("Final save failed: {}", e);
                            }
                        }
                        break;
                    }
                }
            }
        });

        update_tx
    }

    /// Stop auto-saving.
    pub async fn stop(&mut self) -> ForgeResult<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Final save
        let guard = self.session.read().await;
        if let Some(ref session) = *guard {
            self.store.save(session).await?;
        }

        Ok(())
    }

    /// Force an immediate save.
    pub async fn save_now(&self) -> ForgeResult<()> {
        let guard = self.session.read().await;
        if let Some(ref session) = *guard {
            self.store.save(session).await?;
        }
        Ok(())
    }
}
```

---

## Testing Requirements

1. Sessions save and load correctly
2. Compression works for large sessions
3. Atomic writes prevent corruption
4. Metadata cache stays in sync
5. Auto-save triggers at correct intervals
6. Cleanup removes oldest sessions first

---

## Related Specs

- Depends on: [136-forge-session-types.md](136-forge-session-types.md)
- Depends on: [137-forge-config.md](137-forge-config.md)
- Next: [151-forge-resume.md](151-forge-resume.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md), [153-forge-cli.md](153-forge-cli.md)
