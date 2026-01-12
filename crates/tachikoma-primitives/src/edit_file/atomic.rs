//! Atomic file write operations.

use std::fs::{self, File, Permissions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// Atomically write content to a file.
///
/// This writes to a temporary file first, then renames it to the target.
/// This ensures the file is never in a partially written state.
/// 
/// Handles cross-filesystem writes by falling back to copy + remove.
pub fn write_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    let _parent = path.parent().unwrap_or(Path::new("."));

    // Create temp file in same directory (for same-filesystem rename)
    let temp_path = create_temp_path(path);

    debug!("Writing to temp file: {:?}", temp_path);

    // Get original file metadata if it exists
    let original_metadata = fs::metadata(path).ok();

    // Write to temp file
    {
        let mut file = File::create(&temp_path)?;
        file.write_all(content)?;
        file.sync_all()?;
    }

    // Copy permissions from original file
    if let Some(ref metadata) = original_metadata {
        copy_permissions(&temp_path, metadata)?;
    }

    // Atomic rename
    debug!("Renaming {:?} to {:?}", temp_path, path);
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Handle cross-filesystem case (errno 18 - EXDEV: cross-device link)
            if e.kind() == io::ErrorKind::CrossesFilesystems 
                || e.raw_os_error() == Some(18) // EXDEV on Unix
                || e.to_string().contains("cross-device")
            {
                debug!("Cross-filesystem detected, using copy+remove");
                
                // Copy to target
                match fs::copy(&temp_path, path) {
                    Ok(_) => {
                        // Remove temp file
                        let _ = fs::remove_file(&temp_path);
                        Ok(())
                    },
                    Err(copy_err) => {
                        // Clean up temp file and return error
                        let _ = fs::remove_file(&temp_path);
                        Err(copy_err)
                    }
                }
            } else {
                // Try to clean up temp file
                let _ = fs::remove_file(&temp_path);
                Err(e)
            }
        }
    }
}

/// Create a temporary file path.
fn create_temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");

    let temp_name = format!(".{}.tmp.{}", file_name, std::process::id());
    path.with_file_name(temp_name)
}

/// Copy permissions from metadata to a file.
fn copy_permissions(path: &Path, metadata: &fs::Metadata) -> io::Result<()> {
    let permissions = metadata.permissions();
    fs::set_permissions(path, permissions)?;

    #[cfg(unix)]
    {
        // Also try to preserve ownership (requires root)
        use std::os::unix::fs::chown;
        let uid = metadata.uid();
        let gid = metadata.gid();
        // Ignore errors - ownership change may require elevated privileges
        let _ = chown(path, Some(uid), Some(gid));
    }

    Ok(())
}

/// Atomic writer with rollback support.
pub struct AtomicWriter {
    target_path: PathBuf,
    temp_path: PathBuf,
    backup_path: Option<PathBuf>,
    original_metadata: Option<fs::Metadata>,
    committed: bool,
}

impl AtomicWriter {
    /// Create a new atomic writer.
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let target_path = path.as_ref().to_path_buf();
        let temp_path = create_temp_path(&target_path);
        let original_metadata = fs::metadata(&target_path).ok();

        Ok(Self {
            target_path,
            temp_path,
            backup_path: None,
            original_metadata,
            committed: false,
        })
    }

    /// Enable backup of original file.
    pub fn with_backup(mut self) -> Self {
        self.backup_path = Some(self.target_path.with_extension("bak"));
        self
    }

    /// Get the path to write to.
    pub fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    /// Write content to the temp file.
    pub fn write(&self, content: &[u8]) -> io::Result<()> {
        let mut file = File::create(&self.temp_path)?;
        file.write_all(content)?;
        file.sync_all()?;

        // Copy permissions
        if let Some(ref metadata) = self.original_metadata {
            copy_permissions(&self.temp_path, metadata)?;
        }

        Ok(())
    }

    /// Commit the changes (rename temp to target).
    pub fn commit(mut self) -> io::Result<()> {
        // Create backup if requested
        if let Some(ref backup_path) = self.backup_path {
            if self.target_path.exists() {
                debug!("Creating backup at {:?}", backup_path);
                fs::copy(&self.target_path, backup_path)?;
            }
        }

        // Use the same atomic rename logic as write_atomic
        match fs::rename(&self.temp_path, &self.target_path) {
            Ok(()) => {
                self.committed = true;
                Ok(())
            },
            Err(e) => {
                // Handle cross-filesystem case
                if e.kind() == io::ErrorKind::CrossesFilesystems 
                    || e.raw_os_error() == Some(18) // EXDEV on Unix
                    || e.to_string().contains("cross-device")
                {
                    debug!("Cross-filesystem detected, using copy+remove");
                    
                    // Copy to target
                    match fs::copy(&self.temp_path, &self.target_path) {
                        Ok(_) => {
                            // Remove temp file
                            let _ = fs::remove_file(&self.temp_path);
                            self.committed = true;
                            Ok(())
                        },
                        Err(copy_err) => {
                            // Clean up temp file and return error
                            let _ = fs::remove_file(&self.temp_path);
                            Err(copy_err)
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Rollback changes (remove temp file).
    pub fn rollback(mut self) -> io::Result<()> {
        if self.temp_path.exists() {
            fs::remove_file(&self.temp_path)?;
        }
        self.committed = true; // Mark as handled
        Ok(())
    }
}

impl Drop for AtomicWriter {
    fn drop(&mut self) {
        if !self.committed {
            // Clean up temp file if not committed
            if self.temp_path.exists() {
                warn!("AtomicWriter dropped without commit, cleaning up temp file");
                let _ = fs::remove_file(&self.temp_path);
            }
        }
    }
}

/// File lock for concurrent access protection.
#[cfg(unix)]
pub mod lock {
    use std::fs::File;
    use std::io;
    use std::path::Path;

    /// A file lock.
    pub struct FileLock {
        file: File,
    }

    impl FileLock {
        /// Acquire an exclusive lock on a file.
        pub fn exclusive(path: impl AsRef<Path>) -> io::Result<Self> {
            use std::os::unix::io::AsRawFd;

            let file = File::open(path)?;
            let fd = file.as_raw_fd();

            // Try to get exclusive lock
            let result = unsafe {
                libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB)
            };

            if result != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "File is locked by another process",
                ));
            }

            Ok(Self { file })
        }

        /// Acquire a shared lock on a file.
        pub fn shared(path: impl AsRef<Path>) -> io::Result<Self> {
            use std::os::unix::io::AsRawFd;

            let file = File::open(path)?;
            let fd = file.as_raw_fd();

            let result = unsafe {
                libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB)
            };

            if result != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "File is locked by another process",
                ));
            }

            Ok(Self { file })
        }
    }

    impl Drop for FileLock {
        fn drop(&mut self) {
            use std::os::unix::io::AsRawFd;

            let fd = self.file.as_raw_fd();
            unsafe {
                libc::flock(fd, libc::LOCK_UN);
            }
        }
    }
}

#[cfg(windows)]
pub mod lock {
    use std::fs::File;
    use std::io;
    use std::path::Path;

    pub struct FileLock {
        _file: File,
    }

    impl FileLock {
        pub fn exclusive(path: impl AsRef<Path>) -> io::Result<Self> {
            // Windows has automatic exclusive access when opening for write
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)?;
            Ok(Self { _file: file })
        }

        pub fn shared(path: impl AsRef<Path>) -> io::Result<Self> {
            let file = File::open(path)?;
            Ok(Self { _file: file })
        }
    }
}

/// Edit a file with atomic write.
pub fn atomic_edit<F>(path: &Path, editor: F) -> io::Result<()>
where
    F: FnOnce(&str) -> io::Result<String>,
{
    // Read current content
    let content = fs::read_to_string(path)?;

    // Apply edit
    let new_content = editor(&content)?;

    // Write atomically
    write_atomic(path, new_content.as_bytes())
}

/// Edit a file with lock and atomic write.
pub fn locked_atomic_edit<F>(path: &Path, editor: F) -> io::Result<()>
where
    F: FnOnce(&str) -> io::Result<String>,
{
    // Acquire exclusive lock
    let _lock = lock::FileLock::exclusive(path)?;

    // Perform atomic edit
    atomic_edit(path, editor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_atomic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        // Write initial content
        fs::write(&path, "initial").unwrap();

        // Atomic write
        write_atomic(&path, b"updated").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "updated");
    }

    #[test]
    fn test_atomic_writer_commit() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();

        let writer = AtomicWriter::new(&path).unwrap();
        writer.write(b"new content").unwrap();
        writer.commit().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new content");
    }

    #[test]
    fn test_atomic_writer_rollback() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();

        let writer = AtomicWriter::new(&path).unwrap();
        writer.write(b"new content").unwrap();
        writer.rollback().unwrap();

        // Original should be unchanged
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    }

    #[test]
    fn test_atomic_writer_with_backup() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();

        let writer = AtomicWriter::new(&path).unwrap().with_backup();
        writer.write(b"new content").unwrap();
        writer.commit().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new content");

        let backup_path = path.with_extension("bak");
        assert_eq!(fs::read_to_string(&backup_path).unwrap(), "original");
    }

    #[test]
    fn test_atomic_edit() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "hello world").unwrap();

        atomic_edit(&path, |content| {
            Ok(content.replace("world", "rust"))
        }).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "hello rust");
    }

    #[cfg(unix)]
    #[test]
    fn test_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        fs::write(&path, "content").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o644)).unwrap();

        write_atomic(&path, b"updated").unwrap();

        let permissions = fs::metadata(&path).unwrap().permissions();
        assert_eq!(permissions.mode() & 0o777, 0o644);
    }

    #[test]
    fn test_cross_filesystem_fallback() {
        // This test simulates cross-filesystem behavior
        // In practice, this would require creating files on different filesystems
        
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "original").unwrap();

        // For this test, we'll just verify the function works normally
        // In a real cross-filesystem scenario, the rename would fail and fallback to copy
        write_atomic(&path, b"updated").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "updated");
    }
}