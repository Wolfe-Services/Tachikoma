# Spec 116: Spec Directory Structure

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 116
- **Status**: Planned
- **Dependencies**: 001-project-structure, 044-workspace-discovery
- **Estimated Context**: ~10%

## Objective

Define the canonical directory structure for the Tachikoma spec system (THE PIN - Tachikoma Hierarchical Engineering Protocol for Intelligent Navigation). This spec establishes the organization of spec files, phases, templates, and generated artifacts that enable systematic project development through AI-assisted workflows.

## Acceptance Criteria

- [ ] Spec directory structure is well-defined and discoverable
- [ ] Phase-based organization supports logical grouping
- [ ] Templates directory contains all spec templates
- [ ] Generated artifacts have dedicated output locations
- [ ] Index files enable fast spec lookup
- [ ] Nested project specs are properly isolated
- [ ] Symlinks and includes are handled correctly
- [ ] Directory watching detects structure changes

## Implementation Details

### Directory Structure Definition

```rust
// src/spec/directory.rs

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::fs;

/// The canonical spec directory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDirectory {
    /// Root path of the spec system
    pub root: PathBuf,
    /// Phase directories (phase-01-specs, phase-02-specs, etc.)
    pub phases: Vec<PhaseDirectory>,
    /// Templates directory
    pub templates: PathBuf,
    /// Generated artifacts directory
    pub generated: PathBuf,
    /// Index files location
    pub index: PathBuf,
    /// Configuration file path
    pub config: PathBuf,
}

/// A phase directory containing related specs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDirectory {
    /// Phase number (1-based)
    pub number: u32,
    /// Phase name/description
    pub name: String,
    /// Directory path
    pub path: PathBuf,
    /// Spec files in this phase
    pub specs: Vec<SpecFileInfo>,
    /// README for this phase
    pub readme: Option<PathBuf>,
}

/// Basic info about a spec file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFileInfo {
    /// Spec ID (e.g., 116)
    pub id: u32,
    /// Filename (e.g., "116-spec-directory.md")
    pub filename: String,
    /// Full path
    pub path: PathBuf,
    /// Title extracted from file
    pub title: Option<String>,
    /// File modification time
    pub modified: Option<u64>,
}

/// Standard directory names
pub mod names {
    pub const SPECS_ROOT: &str = "specs";
    pub const TEMPLATES: &str = "templates";
    pub const GENERATED: &str = ".generated";
    pub const INDEX: &str = ".index";
    pub const CONFIG: &str = "specs.toml";
    pub const README: &str = "README.md";
    pub const PHASE_PREFIX: &str = "phase-";
    pub const PHASE_SUFFIX: &str = "-specs";
}

impl SpecDirectory {
    /// Discover spec directory structure from a root path
    pub async fn discover(workspace_root: &Path) -> Result<Self, SpecDirectoryError> {
        let specs_root = workspace_root.join(names::SPECS_ROOT);

        if !specs_root.exists() {
            return Err(SpecDirectoryError::NotFound(specs_root));
        }

        let mut phases = Vec::new();
        let mut entries = fs::read_dir(&specs_root).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                if let Some(phase) = Self::parse_phase_dir(&path).await? {
                    phases.push(phase);
                }
            }
        }

        // Sort phases by number
        phases.sort_by_key(|p| p.number);

        Ok(Self {
            root: specs_root.clone(),
            phases,
            templates: specs_root.join(names::TEMPLATES),
            generated: specs_root.join(names::GENERATED),
            index: specs_root.join(names::INDEX),
            config: specs_root.join(names::CONFIG),
        })
    }

    /// Parse a phase directory
    async fn parse_phase_dir(path: &Path) -> Result<Option<PhaseDirectory>, SpecDirectoryError> {
        let dirname = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SpecDirectoryError::InvalidPath(path.to_path_buf()))?;

        // Check if it matches phase-XX-specs pattern
        if !dirname.starts_with(names::PHASE_PREFIX) || !dirname.ends_with(names::PHASE_SUFFIX) {
            return Ok(None);
        }

        // Extract phase number
        let number_str = dirname
            .strip_prefix(names::PHASE_PREFIX)
            .and_then(|s| s.strip_suffix(names::PHASE_SUFFIX))
            .ok_or_else(|| SpecDirectoryError::InvalidPhaseName(dirname.to_string()))?;

        let number: u32 = number_str.parse()
            .map_err(|_| SpecDirectoryError::InvalidPhaseName(dirname.to_string()))?;

        // Collect spec files
        let mut specs = Vec::new();
        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let file_path = entry.path();
            if let Some(spec_info) = Self::parse_spec_file(&file_path).await? {
                specs.push(spec_info);
            }
        }

        // Sort specs by ID
        specs.sort_by_key(|s| s.id);

        // Check for README
        let readme_path = path.join(names::README);
        let readme = if readme_path.exists() { Some(readme_path) } else { None };

        Ok(Some(PhaseDirectory {
            number,
            name: Self::phase_name(number),
            path: path.to_path_buf(),
            specs,
            readme,
        }))
    }

    /// Parse a spec file for basic info
    async fn parse_spec_file(path: &Path) -> Result<Option<SpecFileInfo>, SpecDirectoryError> {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SpecDirectoryError::InvalidPath(path.to_path_buf()))?;

        // Must be a markdown file
        if !filename.ends_with(".md") || filename == names::README {
            return Ok(None);
        }

        // Parse spec ID from filename (e.g., "116-spec-directory.md")
        let id_str = filename.split('-').next()
            .ok_or_else(|| SpecDirectoryError::InvalidSpecFilename(filename.to_string()))?;

        let id: u32 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => return Ok(None), // Not a spec file
        };

        // Get modification time
        let metadata = fs::metadata(path).await?;
        let modified = metadata.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        // Extract title from first line (lazy - full parsing elsewhere)
        let title = Self::extract_title(path).await.ok().flatten();

        Ok(Some(SpecFileInfo {
            id,
            filename: filename.to_string(),
            path: path.to_path_buf(),
            title,
            modified,
        }))
    }

    /// Extract title from spec file's first heading
    async fn extract_title(path: &Path) -> Result<Option<String>, SpecDirectoryError> {
        let content = fs::read_to_string(path).await?;

        for line in content.lines().take(5) {
            if line.starts_with("# ") {
                return Ok(Some(line[2..].trim().to_string()));
            }
        }

        Ok(None)
    }

    /// Get standard phase name
    fn phase_name(number: u32) -> String {
        match number {
            1 => "Foundation".to_string(),
            2 => "Core Intelligence".to_string(),
            3 => "Pattern System".to_string(),
            4 => "Context Engine".to_string(),
            5 => "Advanced Features".to_string(),
            6 => "Spec System".to_string(),
            7 => "Integration".to_string(),
            8 => "Optimization".to_string(),
            _ => format!("Phase {}", number),
        }
    }

    /// Initialize a new spec directory structure
    pub async fn initialize(workspace_root: &Path) -> Result<Self, SpecDirectoryError> {
        let specs_root = workspace_root.join(names::SPECS_ROOT);

        // Create directories
        fs::create_dir_all(&specs_root).await?;
        fs::create_dir_all(specs_root.join(names::TEMPLATES)).await?;
        fs::create_dir_all(specs_root.join(names::GENERATED)).await?;
        fs::create_dir_all(specs_root.join(names::INDEX)).await?;

        // Create default config
        let config_path = specs_root.join(names::CONFIG);
        if !config_path.exists() {
            fs::write(&config_path, Self::default_config()).await?;
        }

        // Create root README
        let readme_path = specs_root.join(names::README);
        if !readme_path.exists() {
            fs::write(&readme_path, Self::default_readme()).await?;
        }

        Self::discover(workspace_root).await
    }

    /// Get path for a new phase directory
    pub fn phase_path(&self, phase_number: u32) -> PathBuf {
        self.root.join(format!("{}{:02}{}",
            names::PHASE_PREFIX,
            phase_number,
            names::PHASE_SUFFIX
        ))
    }

    /// Get path for a new spec file
    pub fn spec_path(&self, phase_number: u32, spec_id: u32, slug: &str) -> PathBuf {
        self.phase_path(phase_number).join(format!("{:03}-{}.md", spec_id, slug))
    }

    /// Find spec by ID
    pub fn find_spec(&self, spec_id: u32) -> Option<&SpecFileInfo> {
        for phase in &self.phases {
            if let Some(spec) = phase.specs.iter().find(|s| s.id == spec_id) {
                return Some(spec);
            }
        }
        None
    }

    /// Get all specs as a flat list
    pub fn all_specs(&self) -> Vec<&SpecFileInfo> {
        self.phases.iter()
            .flat_map(|p| p.specs.iter())
            .collect()
    }

    fn default_config() -> &'static str {
        r#"# Tachikoma Spec System Configuration

[spec]
# Spec ID ranges by phase
phase_ranges = [
    [1, 1, 25],      # Phase 1: 001-025
    [2, 26, 50],     # Phase 2: 026-050
    [3, 51, 75],     # Phase 3: 051-075
    [4, 76, 100],    # Phase 4: 076-100
    [5, 101, 115],   # Phase 5: 101-115
    [6, 116, 135],   # Phase 6: 116-135
    [7, 136, 155],   # Phase 7: 136-155
    [8, 156, 175],   # Phase 8: 156-175
]

[templates]
# Available templates
types = ["feature", "component", "integration", "refactor", "test"]

[validation]
# Validation rules
require_metadata = true
require_acceptance_criteria = true
require_implementation = true
require_tests = true

[generation]
# Auto-generation settings
auto_readme = true
auto_index = true
watch_changes = true
"#
    }

    fn default_readme() -> &'static str {
        r#"# Tachikoma Specifications

THE PIN - Tachikoma Hierarchical Engineering Protocol for Intelligent Navigation

## Overview

This directory contains the complete specification system for Tachikoma development.

## Directory Structure

```
specs/
├── README.md              # This file
├── specs.toml             # Configuration
├── templates/             # Spec templates
├── .generated/            # Auto-generated artifacts
├── .index/                # Search indexes
├── phase-01-specs/        # Foundation
├── phase-02-specs/        # Core Intelligence
├── phase-03-specs/        # Pattern System
├── phase-04-specs/        # Context Engine
├── phase-05-specs/        # Advanced Features
├── phase-06-specs/        # Spec System
├── phase-07-specs/        # Integration
└── phase-08-specs/        # Optimization
```

## Usage

Specs can be referenced by ID (e.g., `spec:116`) or by path.
"#
    }
}

/// Errors for spec directory operations
#[derive(Debug, thiserror::Error)]
pub enum SpecDirectoryError {
    #[error("Spec directory not found: {0}")]
    NotFound(PathBuf),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Invalid phase directory name: {0}")]
    InvalidPhaseName(String),

    #[error("Invalid spec filename: {0}")]
    InvalidSpecFilename(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize_spec_directory() {
        let temp = TempDir::new().unwrap();
        let dir = SpecDirectory::initialize(temp.path()).await.unwrap();

        assert!(dir.root.exists());
        assert!(dir.templates.exists());
        assert!(dir.config.exists());
    }

    #[tokio::test]
    async fn test_phase_path_generation() {
        let temp = TempDir::new().unwrap();
        let dir = SpecDirectory::initialize(temp.path()).await.unwrap();

        let phase_path = dir.phase_path(6);
        assert!(phase_path.to_string_lossy().contains("phase-06-specs"));
    }

    #[tokio::test]
    async fn test_spec_path_generation() {
        let temp = TempDir::new().unwrap();
        let dir = SpecDirectory::initialize(temp.path()).await.unwrap();

        let spec_path = dir.spec_path(6, 116, "spec-directory");
        assert!(spec_path.to_string_lossy().contains("116-spec-directory.md"));
    }
}
```

### Directory Watcher

```rust
// src/spec/watcher.rs

use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Events from spec directory watching
#[derive(Debug, Clone)]
pub enum SpecDirectoryEvent {
    SpecCreated(PathBuf),
    SpecModified(PathBuf),
    SpecDeleted(PathBuf),
    PhaseCreated(PathBuf),
    PhaseDeleted(PathBuf),
    ConfigChanged,
    TemplateChanged(PathBuf),
}

/// Watch spec directory for changes
pub struct SpecDirectoryWatcher {
    watcher: notify::RecommendedWatcher,
    receiver: mpsc::Receiver<SpecDirectoryEvent>,
}

impl SpecDirectoryWatcher {
    pub fn new(spec_root: PathBuf) -> Result<Self, notify::Error> {
        let (tx, rx) = mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if let Some(spec_event) = Self::classify_event(&event, &spec_root) {
                    let _ = tx.blocking_send(spec_event);
                }
            }
        })?;

        watcher.watch(&spec_root, RecursiveMode::Recursive)?;

        Ok(Self { watcher, receiver: rx })
    }

    fn classify_event(event: &Event, root: &PathBuf) -> Option<SpecDirectoryEvent> {
        let path = event.paths.first()?;

        match &event.kind {
            EventKind::Create(_) => {
                if path.is_dir() && path.file_name()?.to_str()?.starts_with("phase-") {
                    Some(SpecDirectoryEvent::PhaseCreated(path.clone()))
                } else if path.extension()? == "md" {
                    Some(SpecDirectoryEvent::SpecCreated(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Modify(_) => {
                if path.file_name()?.to_str()? == "specs.toml" {
                    Some(SpecDirectoryEvent::ConfigChanged)
                } else if path.extension()? == "md" {
                    Some(SpecDirectoryEvent::SpecModified(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Remove(_) => {
                if path.extension()? == "md" {
                    Some(SpecDirectoryEvent::SpecDeleted(path.clone()))
                } else {
                    None
                }
            }
            _ => None
        }
    }

    pub async fn next_event(&mut self) -> Option<SpecDirectoryEvent> {
        self.receiver.recv().await
    }
}
```

## Testing Requirements

- [ ] Unit tests for directory discovery
- [ ] Unit tests for phase parsing
- [ ] Unit tests for spec file parsing
- [ ] Integration tests for directory initialization
- [ ] Tests for directory watching
- [ ] Tests for path generation utilities
- [ ] Tests for nested directory handling
- [ ] Performance tests for large spec sets

## Related Specs

- **117-spec-templates.md**: Template system using this directory structure
- **118-readme-lookup.md**: README lookup within spec directories
- **119-readme-autogen.md**: Auto-generation of phase READMEs
- **131-spec-search-index.md**: Index building from directory structure
