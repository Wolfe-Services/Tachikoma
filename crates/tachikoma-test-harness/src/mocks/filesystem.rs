//! Mock file system for testing file operations.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use async_trait::async_trait;

/// File metadata
#[derive(Debug, Clone)]
pub struct MockMetadata {
    pub is_file: bool,
    pub is_dir: bool,
    pub len: u64,
    pub readonly: bool,
    pub modified: SystemTime,
    pub created: SystemTime,
}

impl MockMetadata {
    pub fn file(len: u64) -> Self {
        let now = SystemTime::now();
        Self {
            is_file: true,
            is_dir: false,
            len,
            readonly: false,
            modified: now,
            created: now,
        }
    }

    pub fn dir() -> Self {
        let now = SystemTime::now();
        Self {
            is_file: false,
            is_dir: true,
            len: 0,
            readonly: false,
            modified: now,
            created: now,
        }
    }
}

/// A file system entry
#[derive(Debug, Clone)]
pub enum FsEntry {
    File {
        content: Vec<u8>,
        metadata: MockMetadata,
    },
    Directory {
        metadata: MockMetadata,
    },
    Symlink {
        target: PathBuf,
    },
}

impl FsEntry {
    pub fn file(content: impl Into<Vec<u8>>) -> Self {
        let content = content.into();
        FsEntry::File {
            metadata: MockMetadata::file(content.len() as u64),
            content,
        }
    }

    pub fn text_file(content: impl Into<String>) -> Self {
        Self::file(content.into().into_bytes())
    }

    pub fn directory() -> Self {
        FsEntry::Directory {
            metadata: MockMetadata::dir(),
        }
    }

    pub fn symlink(target: impl Into<PathBuf>) -> Self {
        FsEntry::Symlink {
            target: target.into(),
        }
    }
}

/// Mock file system state
#[derive(Debug, Clone, Default)]
pub struct MockFileSystem {
    entries: Arc<RwLock<HashMap<PathBuf, FsEntry>>>,
    /// Track all operations for verification
    operations: Arc<RwLock<Vec<FsOperation>>>,
}

/// Recorded file system operation
#[derive(Debug, Clone)]
pub enum FsOperation {
    Read(PathBuf),
    Write(PathBuf, usize),
    Delete(PathBuf),
    CreateDir(PathBuf),
    Rename(PathBuf, PathBuf),
    Copy(PathBuf, PathBuf),
    Metadata(PathBuf),
    ReadDir(PathBuf),
    Exists(PathBuf),
}

impl MockFileSystem {
    /// Create a new empty mock file system
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial files
    pub fn with_files(files: impl IntoIterator<Item = (impl Into<PathBuf>, FsEntry)>) -> Self {
        let fs = Self::new();
        {
            let mut entries = fs.entries.write().unwrap();
            for (path, entry) in files {
                entries.insert(path.into(), entry);
            }
        }
        fs
    }

    /// Add a file to the filesystem
    pub fn add_file(&self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) {
        let path = path.into();
        let mut entries = self.entries.write().unwrap();

        // Create parent directories
        if let Some(parent) = path.parent() {
            self.ensure_parent_dirs(&mut entries, parent);
        }

        entries.insert(path, FsEntry::file(content));
    }

    /// Add a text file
    pub fn add_text_file(&self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.add_file(path, content.into().into_bytes());
    }

    /// Add a directory
    pub fn add_dir(&self, path: impl Into<PathBuf>) {
        let mut entries = self.entries.write().unwrap();
        entries.insert(path.into(), FsEntry::directory());
    }

    /// Check if path exists
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        self.record_op(FsOperation::Exists(path.as_ref().to_path_buf()));
        self.entries.read().unwrap().contains_key(path.as_ref())
    }

    /// Read file contents
    pub fn read(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let path = path.as_ref();
        self.record_op(FsOperation::Read(path.to_path_buf()));

        let entries = self.entries.read().unwrap();
        match entries.get(path) {
            Some(FsEntry::File { content, metadata }) => {
                if metadata.readonly {
                    // Still readable even if readonly
                }
                Ok(content.clone())
            }
            Some(FsEntry::Directory { .. }) => {
                Err(io::Error::new(io::ErrorKind::InvalidInput, "Is a directory"))
            }
            Some(FsEntry::Symlink { target }) => {
                drop(entries);
                self.read(target)
            }
            None => Err(io::Error::new(io::ErrorKind::NotFound, "File not found")),
        }
    }

    /// Read file as string
    pub fn read_to_string(&self, path: impl AsRef<Path>) -> io::Result<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Write file contents
    pub fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> io::Result<()> {
        let path = path.as_ref();
        let content = content.as_ref();
        self.record_op(FsOperation::Write(path.to_path_buf(), content.len()));

        let mut entries = self.entries.write().unwrap();

        // Check if file is readonly
        if let Some(FsEntry::File { metadata, .. }) = entries.get(path) {
            if metadata.readonly {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "File is readonly",
                ));
            }
        }

        // Create parent directories
        if let Some(parent) = path.parent() {
            self.ensure_parent_dirs(&mut entries, parent);
        }

        entries.insert(path.to_path_buf(), FsEntry::file(content.to_vec()));
        Ok(())
    }

    /// Delete a file or directory
    pub fn remove(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        self.record_op(FsOperation::Delete(path.to_path_buf()));

        let mut entries = self.entries.write().unwrap();

        if entries.remove(path).is_none() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        }

        Ok(())
    }

    /// Create a directory
    pub fn create_dir(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        self.record_op(FsOperation::CreateDir(path.to_path_buf()));

        let mut entries = self.entries.write().unwrap();

        if entries.contains_key(path) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Already exists"));
        }

        entries.insert(path.to_path_buf(), FsEntry::directory());
        Ok(())
    }

    /// Create directory and all parents
    pub fn create_dir_all(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let mut entries = self.entries.write().unwrap();
        self.ensure_parent_dirs(&mut entries, path);
        entries.insert(path.to_path_buf(), FsEntry::directory());
        Ok(())
    }

    /// Read directory entries
    pub fn read_dir(&self, path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
        let path = path.as_ref();
        self.record_op(FsOperation::ReadDir(path.to_path_buf()));

        let entries = self.entries.read().unwrap();

        // Check if path is a directory
        match entries.get(path) {
            Some(FsEntry::Directory { .. }) => {}
            Some(_) => {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a directory"))
            }
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Directory not found")),
        }

        let prefix = if path == Path::new("") || path == Path::new("/") {
            PathBuf::new()
        } else {
            path.to_path_buf()
        };

        let children: Vec<PathBuf> = entries
            .keys()
            .filter(|p| {
                if let Some(parent) = p.parent() {
                    parent == path
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(children)
    }

    /// Get file metadata
    pub fn metadata(&self, path: impl AsRef<Path>) -> io::Result<MockMetadata> {
        let path = path.as_ref();
        self.record_op(FsOperation::Metadata(path.to_path_buf()));

        let entries = self.entries.read().unwrap();
        match entries.get(path) {
            Some(FsEntry::File { metadata, .. }) => Ok(metadata.clone()),
            Some(FsEntry::Directory { metadata }) => Ok(metadata.clone()),
            Some(FsEntry::Symlink { target }) => {
                drop(entries);
                self.metadata(target)
            }
            None => Err(io::Error::new(io::ErrorKind::NotFound, "Not found")),
        }
    }

    /// Set file as readonly
    pub fn set_readonly(&self, path: impl AsRef<Path>, readonly: bool) -> io::Result<()> {
        let path = path.as_ref();
        let mut entries = self.entries.write().unwrap();

        match entries.get_mut(path) {
            Some(FsEntry::File { metadata, .. }) => {
                metadata.readonly = readonly;
                Ok(())
            }
            Some(FsEntry::Directory { metadata }) => {
                metadata.readonly = readonly;
                Ok(())
            }
            _ => Err(io::Error::new(io::ErrorKind::NotFound, "Not found")),
        }
    }

    /// Get recorded operations
    pub fn operations(&self) -> Vec<FsOperation> {
        self.operations.read().unwrap().clone()
    }

    /// Clear recorded operations
    pub fn clear_operations(&self) {
        self.operations.write().unwrap().clear();
    }

    /// Get all file paths
    pub fn all_paths(&self) -> Vec<PathBuf> {
        self.entries.read().unwrap().keys().cloned().collect()
    }

    // Internal helpers

    fn ensure_parent_dirs(&self, entries: &mut HashMap<PathBuf, FsEntry>, path: &Path) {
        let mut current = PathBuf::new();
        for component in path.components() {
            current.push(component);
            entries
                .entry(current.clone())
                .or_insert_with(FsEntry::directory);
        }
    }

    fn record_op(&self, op: FsOperation) {
        self.operations.write().unwrap().push(op);
    }
}

/// Abstraction over file system operations for dependency injection
#[async_trait]
pub trait FileSystem: Send + Sync {
    async fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    async fn read_to_string(&self, path: &Path) -> io::Result<String>;
    async fn write(&self, path: &Path, content: &[u8]) -> io::Result<()>;
    async fn exists(&self, path: &Path) -> bool;
    async fn remove(&self, path: &Path) -> io::Result<()>;
    async fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    async fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;
    async fn metadata(&self, path: &Path) -> io::Result<MockMetadata>;
}

/// Real file system implementation
pub struct RealFileSystem;

#[async_trait]
impl FileSystem for RealFileSystem {
    async fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        tokio::fs::read(path).await
    }

    async fn read_to_string(&self, path: &Path) -> io::Result<String> {
        tokio::fs::read_to_string(path).await
    }

    async fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        tokio::fs::write(path, content).await
    }

    async fn exists(&self, path: &Path) -> bool {
        tokio::fs::try_exists(path).await.unwrap_or(false)
    }

    async fn remove(&self, path: &Path) -> io::Result<()> {
        tokio::fs::remove_file(path).await
    }

    async fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        tokio::fs::create_dir_all(path).await
    }

    async fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;
        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry.path());
        }
        Ok(entries)
    }

    async fn metadata(&self, path: &Path) -> io::Result<MockMetadata> {
        let metadata = tokio::fs::metadata(path).await?;
        Ok(MockMetadata {
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            len: metadata.len(),
            readonly: metadata.permissions().readonly(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
        })
    }
}

/// Mock file system implementing the async trait
#[async_trait]
impl FileSystem for MockFileSystem {
    async fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.read(path)
    }

    async fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.read_to_string(path)
    }

    async fn write(&self, path: &Path, content: &[u8]) -> io::Result<()> {
        self.write(path, content)
    }

    async fn exists(&self, path: &Path) -> bool {
        self.exists(path)
    }

    async fn remove(&self, path: &Path) -> io::Result<()> {
        self.remove(path)
    }

    async fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        self.create_dir_all(path)
    }

    async fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.read_dir(path)
    }

    async fn metadata(&self, path: &Path) -> io::Result<MockMetadata> {
        self.metadata(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_fs_read_write() {
        let fs = MockFileSystem::new();

        fs.write("/test.txt", b"Hello, world!").unwrap();
        let content = fs.read_to_string("/test.txt").unwrap();

        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_mock_fs_not_found() {
        let fs = MockFileSystem::new();

        let result = fs.read("/nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_fs_readonly() {
        let fs = MockFileSystem::new();

        fs.write("/readonly.txt", b"content").unwrap();
        fs.set_readonly("/readonly.txt", true).unwrap();

        let result = fs.write("/readonly.txt", b"new content");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_fs_tracks_operations() {
        let fs = MockFileSystem::new();

        fs.write("/file.txt", b"data").unwrap();
        fs.read("/file.txt").unwrap();

        let ops = fs.operations();
        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_directory_operations() {
        let fs = MockFileSystem::new();

        fs.create_dir("/testdir").unwrap();
        fs.write("/testdir/file.txt", b"content").unwrap();

        let children = fs.read_dir("/testdir").unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], PathBuf::from("/testdir/file.txt"));
    }

    #[test]
    fn test_with_files_creation() {
        let fs = MockFileSystem::with_files([
            ("/file1.txt", FsEntry::text_file("content1")),
            ("/file2.txt", FsEntry::text_file("content2")),
            ("/dir", FsEntry::directory()),
        ]);

        assert!(fs.exists("/file1.txt"));
        assert!(fs.exists("/file2.txt"));
        assert!(fs.exists("/dir"));
        
        let content1 = fs.read_to_string("/file1.txt").unwrap();
        assert_eq!(content1, "content1");
    }

    #[test]
    fn test_metadata() {
        let fs = MockFileSystem::new();
        fs.write("/test.txt", b"hello").unwrap();

        let metadata = fs.metadata("/test.txt").unwrap();
        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
        assert_eq!(metadata.len, 5);
        assert!(!metadata.readonly);
    }

    #[test]
    fn test_symlink() {
        let fs = MockFileSystem::new();
        
        // Create a file
        fs.write("/target.txt", b"target content").unwrap();
        
        // Add symlink manually
        {
            let mut entries = fs.entries.write().unwrap();
            entries.insert(
                PathBuf::from("/link.txt"),
                FsEntry::symlink("/target.txt")
            );
        }

        // Reading through symlink should work
        let content = fs.read_to_string("/link.txt").unwrap();
        assert_eq!(content, "target content");
    }

    #[test]
    fn test_error_conditions() {
        let fs = MockFileSystem::new();

        // File not found
        assert!(fs.read("/nonexistent").is_err());
        assert!(fs.metadata("/nonexistent").is_err());
        assert!(fs.remove("/nonexistent").is_err());

        // Try to read directory as file
        fs.create_dir("/dir").unwrap();
        assert!(fs.read("/dir").is_err());

        // Try to read_dir on file
        fs.write("/file.txt", b"content").unwrap();
        assert!(fs.read_dir("/file.txt").is_err());

        // Try to create existing directory
        assert!(fs.create_dir("/dir").is_err());
    }

    #[test]
    fn test_operation_tracking() {
        let fs = MockFileSystem::new();

        fs.write("/test.txt", b"data").unwrap();
        fs.read("/test.txt").unwrap();
        fs.exists("/test.txt");
        fs.metadata("/test.txt").unwrap();
        fs.remove("/test.txt").unwrap();

        let ops = fs.operations();
        assert_eq!(ops.len(), 5);

        // Verify operation types
        assert!(matches!(ops[0], FsOperation::Write(_, 4)));
        assert!(matches!(ops[1], FsOperation::Read(_)));
        assert!(matches!(ops[2], FsOperation::Exists(_)));
        assert!(matches!(ops[3], FsOperation::Metadata(_)));
        assert!(matches!(ops[4], FsOperation::Delete(_)));

        // Test clearing operations
        fs.clear_operations();
        assert_eq!(fs.operations().len(), 0);
    }
}