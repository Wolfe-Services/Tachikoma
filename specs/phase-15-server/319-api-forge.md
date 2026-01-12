# Spec 319: Forge API

## Phase
15 - Server/API Layer

## Spec ID
319

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 313: Route Definitions
- Spec 301: Forge Abstraction

## Estimated Context
~10%

---

## Objective

Implement the Forge API for Tachikoma, providing endpoints to manage forge connections (local filesystem, Git repositories, remote forges), file operations, and repository management.

---

## Acceptance Criteria

- [ ] CRUD operations for forge configurations
- [ ] File system operations (read, write, list, delete)
- [ ] Git operations (status, diff, commit, push)
- [ ] Repository cloning and management
- [ ] Forge health checking
- [ ] File watching configuration
- [ ] Multi-forge support per mission

---

## Implementation Details

### Request/Response Types

```rust
// src/api/types/forge.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;
use std::path::PathBuf;

/// Forge types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForgeType {
    Local,
    Git,
    GitHub,
    GitLab,
    Bitbucket,
}

/// Request to create a forge connection
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateForgeRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    pub forge_type: ForgeType,

    /// Base path for local forge or repository URL for Git
    #[validate(length(min = 1))]
    pub path_or_url: String,

    /// Optional authentication
    pub auth: Option<ForgeAuth>,

    /// Mission to associate with (optional)
    pub mission_id: Option<Uuid>,

    /// Git branch (for Git forges)
    pub branch: Option<String>,

    /// File patterns to watch
    pub watch_patterns: Option<Vec<String>>,

    /// File patterns to ignore
    pub ignore_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ForgeAuth {
    None,
    Token { token: String },
    BasicAuth { username: String, password: String },
    SshKey { key_path: String, passphrase: Option<String> },
}

/// Forge response
#[derive(Debug, Clone, Serialize)]
pub struct ForgeResponse {
    pub id: Uuid,
    pub name: String,
    pub forge_type: ForgeType,
    pub path_or_url: String,
    pub mission_id: Option<Uuid>,
    pub branch: Option<String>,
    pub status: ForgeStatus,
    pub stats: ForgeStats,
    pub watch_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeStatus {
    Connected,
    Disconnected,
    Syncing,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForgeStats {
    pub total_files: i64,
    pub tracked_files: i64,
    pub modified_files: i64,
    pub untracked_files: i64,
}

/// File operations
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified_at: Option<DateTime<Utc>>,
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub encoding: FileEncoding,
    pub size: u64,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileEncoding {
    Utf8,
    Base64,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct WriteFileRequest {
    #[validate(length(min = 1))]
    pub path: String,

    pub content: String,

    #[serde(default)]
    pub encoding: Option<FileEncoding>,

    /// Create parent directories if they don't exist
    #[serde(default)]
    pub create_parents: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct DeleteFileRequest {
    #[validate(length(min = 1))]
    pub path: String,

    /// Recursively delete directories
    #[serde(default)]
    pub recursive: bool,
}

/// Git operations
#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: i32,
    pub behind: i32,
    pub staged: Vec<GitFileStatus>,
    pub unstaged: Vec<GitFileStatus>,
    pub untracked: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: GitChangeType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct GitCommitRequest {
    #[validate(length(min = 1, max = 500))]
    pub message: String,

    /// Files to stage (empty = all)
    pub files: Option<Vec<String>>,

    /// Author name override
    pub author_name: Option<String>,

    /// Author email override
    pub author_email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitCommitResponse {
    pub commit_hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub files_changed: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitDiffRequest {
    /// Compare against this ref (default: HEAD)
    pub base: Option<String>,

    /// Specific files to diff
    pub files: Option<Vec<String>>,

    /// Include context lines
    #[serde(default = "default_context_lines")]
    pub context_lines: u32,
}

fn default_context_lines() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize)]
pub struct GitDiff {
    pub files: Vec<FileDiff>,
    pub stats: DiffStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub status: GitChangeType,
    pub hunks: Vec<DiffHunk>,
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffHunk {
    pub old_start: i32,
    pub old_lines: i32,
    pub new_start: i32,
    pub new_lines: i32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffStats {
    pub files_changed: i32,
    pub insertions: i32,
    pub deletions: i32,
}
```

### Forge Handlers

```rust
// src/server/handlers/forge.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::api::types::forge::*;
use crate::server::error::{ApiError, ApiResult};
use crate::server::state::AppState;
use crate::forge::{Forge, ForgeConfig, LocalForge, GitForge};

/// List all forge connections
pub async fn list_forges(
    State(state): State<AppState>,
    Query(params): Query<ForgeListParams>,
) -> ApiResult<Json<Vec<ForgeResponse>>> {
    let registry = state.forge_registry().read().await;

    let forges: Vec<ForgeResponse> = if let Some(mission_id) = params.mission_id {
        registry
            .list_for_mission(mission_id)
            .into_iter()
            .map(|f| f.into())
            .collect()
    } else {
        registry
            .list_all()
            .into_iter()
            .map(|f| f.into())
            .collect()
    };

    Ok(Json(forges))
}

/// Create a new forge connection
pub async fn create_forge(
    State(state): State<AppState>,
    Json(request): Json<CreateForgeRequest>,
) -> ApiResult<(StatusCode, Json<ForgeResponse>)> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let forge: Box<dyn Forge> = match request.forge_type {
        ForgeType::Local => {
            let config = ForgeConfig {
                id: Uuid::new_v4(),
                name: request.name.clone(),
                base_path: PathBuf::from(&request.path_or_url),
                watch_patterns: request.watch_patterns.clone().unwrap_or_default(),
                ignore_patterns: request.ignore_patterns.clone().unwrap_or_else(default_ignore_patterns),
            };

            // Verify path exists and is accessible
            if !tokio::fs::metadata(&request.path_or_url).await.is_ok() {
                return Err(ApiError::bad_request(format!(
                    "Path does not exist or is not accessible: {}",
                    request.path_or_url
                )));
            }

            Box::new(LocalForge::new(config).await?)
        }
        ForgeType::Git | ForgeType::GitHub | ForgeType::GitLab | ForgeType::Bitbucket => {
            let config = GitForgeConfig {
                id: Uuid::new_v4(),
                name: request.name.clone(),
                url: request.path_or_url.clone(),
                branch: request.branch.clone().unwrap_or_else(|| "main".to_string()),
                auth: request.auth.clone().map(|a| a.into()),
                watch_patterns: request.watch_patterns.clone().unwrap_or_default(),
                ignore_patterns: request.ignore_patterns.clone().unwrap_or_else(default_ignore_patterns),
            };

            Box::new(GitForge::new(config).await?)
        }
    };

    let id = forge.id();
    let mut registry = state.forge_registry().write().await;

    // Associate with mission if specified
    if let Some(mission_id) = request.mission_id {
        registry.associate_with_mission(id, mission_id);
    }

    registry.register(forge);

    let forge = registry.get(id).unwrap();
    let response: ForgeResponse = forge.into();

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a forge by ID
pub async fn get_forge(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
) -> ApiResult<Json<ForgeResponse>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    Ok(Json(forge.into()))
}

/// Delete a forge connection
pub async fn delete_forge(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let mut registry = state.forge_registry().write().await;

    registry
        .remove(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Check forge health
pub async fn health_check(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
) -> ApiResult<Json<ForgeHealthResponse>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let health = forge.health_check().await?;

    Ok(Json(ForgeHealthResponse {
        forge_id,
        status: health.status,
        latency_ms: health.latency.as_millis() as i64,
        message: health.message,
        checked_at: Utc::now(),
    }))
}

// File operations

/// List files in a directory
pub async fn list_files(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Query(params): Query<ListFilesParams>,
) -> ApiResult<Json<Vec<FileEntry>>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let path = params.path.as_deref().unwrap_or(".");
    let entries = forge.list_files(path).await?;

    Ok(Json(entries.into_iter().map(|e| e.into()).collect()))
}

/// Read file content
pub async fn read_file(
    State(state): State<AppState>,
    Path((forge_id, file_path)): Path<(Uuid, String)>,
) -> ApiResult<Json<FileContent>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let content = forge.read_file(&file_path).await?;

    // Detect encoding
    let (content_str, encoding) = if content.is_ascii() || std::str::from_utf8(&content).is_ok() {
        (String::from_utf8_lossy(&content).to_string(), FileEncoding::Utf8)
    } else {
        (base64::encode(&content), FileEncoding::Base64)
    };

    let metadata = forge.file_metadata(&file_path).await?;

    Ok(Json(FileContent {
        path: file_path,
        content: content_str,
        encoding,
        size: content.len() as u64,
        modified_at: metadata.modified_at,
    }))
}

/// Write file content
pub async fn write_file(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Json(request): Json<WriteFileRequest>,
) -> ApiResult<StatusCode> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    // Decode content if needed
    let content = match request.encoding.unwrap_or(FileEncoding::Utf8) {
        FileEncoding::Utf8 => request.content.into_bytes(),
        FileEncoding::Base64 => base64::decode(&request.content)
            .map_err(|e| ApiError::bad_request(format!("Invalid base64: {}", e)))?,
    };

    // Create parent directories if requested
    if request.create_parents {
        if let Some(parent) = std::path::Path::new(&request.path).parent() {
            forge.create_directory(parent.to_str().unwrap()).await?;
        }
    }

    forge.write_file(&request.path, &content).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Delete a file or directory
pub async fn delete_file(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Json(request): Json<DeleteFileRequest>,
) -> ApiResult<StatusCode> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    if request.recursive {
        forge.delete_directory(&request.path).await?;
    } else {
        forge.delete_file(&request.path).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// Git operations

/// Get Git status
pub async fn git_status(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
) -> ApiResult<Json<GitStatus>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let git_forge = forge
        .as_git()
        .ok_or_else(|| ApiError::bad_request("Forge is not a Git repository"))?;

    let status = git_forge.status().await?;

    Ok(Json(status.into()))
}

/// Get Git diff
pub async fn git_diff(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Query(request): Query<GitDiffRequest>,
) -> ApiResult<Json<GitDiff>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let git_forge = forge
        .as_git()
        .ok_or_else(|| ApiError::bad_request("Forge is not a Git repository"))?;

    let diff = git_forge
        .diff(request.base.as_deref(), request.files.as_deref(), request.context_lines)
        .await?;

    Ok(Json(diff.into()))
}

/// Create Git commit
pub async fn git_commit(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Json(request): Json<GitCommitRequest>,
) -> ApiResult<Json<GitCommitResponse>> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let git_forge = forge
        .as_git()
        .ok_or_else(|| ApiError::bad_request("Forge is not a Git repository"))?;

    // Stage files
    if let Some(files) = request.files {
        git_forge.stage(&files).await?;
    } else {
        git_forge.stage_all().await?;
    }

    // Create commit
    let commit = git_forge
        .commit(
            &request.message,
            request.author_name.as_deref(),
            request.author_email.as_deref(),
        )
        .await?;

    Ok(Json(GitCommitResponse {
        commit_hash: commit.hash,
        message: commit.message,
        author: commit.author,
        timestamp: commit.timestamp,
        files_changed: commit.files_changed,
    }))
}

/// Push to remote
pub async fn git_push(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Json(request): Json<GitPushRequest>,
) -> ApiResult<Json<GitPushResponse>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let git_forge = forge
        .as_git()
        .ok_or_else(|| ApiError::bad_request("Forge is not a Git repository"))?;

    let result = git_forge
        .push(request.remote.as_deref(), request.branch.as_deref(), request.force)
        .await?;

    Ok(Json(GitPushResponse {
        success: result.success,
        remote: result.remote,
        branch: result.branch,
        commits_pushed: result.commits_pushed,
    }))
}

/// Pull from remote
pub async fn git_pull(
    State(state): State<AppState>,
    Path(forge_id): Path<Uuid>,
    Json(request): Json<GitPullRequest>,
) -> ApiResult<Json<GitPullResponse>> {
    let registry = state.forge_registry().read().await;

    let forge = registry
        .get(forge_id)
        .ok_or_else(|| ApiError::not_found_with_id("Forge", forge_id.to_string()))?;

    let git_forge = forge
        .as_git()
        .ok_or_else(|| ApiError::bad_request("Forge is not a Git repository"))?;

    let result = git_forge
        .pull(request.remote.as_deref(), request.branch.as_deref())
        .await?;

    Ok(Json(GitPullResponse {
        success: result.success,
        commits_pulled: result.commits_pulled,
        files_updated: result.files_updated,
        conflicts: result.conflicts,
    }))
}

// Helper functions

fn default_ignore_patterns() -> Vec<String> {
    vec![
        ".git".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        ".DS_Store".to_string(),
        "*.pyc".to_string(),
        "__pycache__".to_string(),
    ]
}
```

### Routes

```rust
// src/server/routes/api/forge.rs
use axum::{
    Router,
    routing::{get, post, put, delete},
};

use crate::server::state::AppState;
use crate::server::handlers::forge as handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Forge management
        .route("/", get(handlers::list_forges).post(handlers::create_forge))
        .route("/:forge_id", get(handlers::get_forge).delete(handlers::delete_forge))
        .route("/:forge_id/health", get(handlers::health_check))
        // File operations
        .route("/:forge_id/files", get(handlers::list_files))
        .route("/:forge_id/files/*path", get(handlers::read_file))
        .route("/:forge_id/files", post(handlers::write_file))
        .route("/:forge_id/files/delete", post(handlers::delete_file))
        // Git operations
        .route("/:forge_id/git/status", get(handlers::git_status))
        .route("/:forge_id/git/diff", get(handlers::git_diff))
        .route("/:forge_id/git/commit", post(handlers::git_commit))
        .route("/:forge_id/git/push", post(handlers::git_push))
        .route("/:forge_id/git/pull", post(handlers::git_pull))
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_local_forge() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state().await;

        let request = CreateForgeRequest {
            name: "Test Forge".to_string(),
            forge_type: ForgeType::Local,
            path_or_url: temp_dir.path().to_str().unwrap().to_string(),
            auth: None,
            mission_id: None,
            branch: None,
            watch_patterns: None,
            ignore_patterns: None,
        };

        let result = create_forge(State(state), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let state = create_test_state().await;

        // Create forge
        let forge_id = create_test_forge(&state, temp_dir.path()).await;

        // Write file
        let write_request = WriteFileRequest {
            path: "test.txt".to_string(),
            content: "Hello, World!".to_string(),
            encoding: None,
            create_parents: false,
        };

        let result = write_file(
            State(state.clone()),
            Path(forge_id),
            Json(write_request),
        ).await;

        assert!(result.is_ok());

        // Read file
        let result = read_file(
            State(state.clone()),
            Path((forge_id, "test.txt".to_string())),
        ).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.content, "Hello, World!");
    }
}
```

---

## Related Specs

- **Spec 301**: Forge Abstraction
- **Spec 317**: Missions API
- **Spec 318**: Specs API
