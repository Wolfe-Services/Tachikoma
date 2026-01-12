//! File system utilities for Tachikoma.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tachikoma_common_core::{Error, ErrorCode, Result};

/// Read a file to string with size limit.
pub fn read_to_string(path: impl AsRef<Path>, max_size: usize) -> Result<String> {
    let path = path.as_ref();

    let metadata = fs::metadata(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => Error::FileSystem {
            code: ErrorCode::FILE_NOT_FOUND,
            message: format!("file not found: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
        io::ErrorKind::PermissionDenied => Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("permission denied: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
        _ => Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("failed to read metadata: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
    })?;

    if metadata.len() as usize > max_size {
        return Err(Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("file too large: {} bytes (max: {})", metadata.len(), max_size),
            path: Some(path.to_string_lossy().to_string()),
            source: None,
        });
    }

    fs::read_to_string(path).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_READ_ERROR,
        message: format!("failed to read file: {}", path.display()),
        path: Some(path.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })
}

/// Read a file to bytes with encoding detection.
pub fn read_with_encoding_detection(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => Error::FileSystem {
            code: ErrorCode::FILE_NOT_FOUND,
            message: format!("file not found: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
        _ => Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("failed to read file: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
    })?;

    // Simple encoding detection - check for UTF-8 BOM, then try UTF-8, fallback to latin-1
    let result = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 BOM - remove BOM and decode as UTF-8
        match String::from_utf8(bytes[3..].to_vec()) {
            Ok(s) => s,
            Err(_) => {
                // Fallback to latin-1
                bytes[3..].iter().map(|&b| b as char).collect()
            }
        }
    } else {
        // Try UTF-8 first
        match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => {
                // Fallback to latin-1 (always succeeds)
                bytes.iter().map(|&b| b as char).collect()
            }
        }
    };

    Ok(result)
}

/// Read a file to bytes.
pub fn read_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => Error::FileSystem {
            code: ErrorCode::FILE_NOT_FOUND,
            message: format!("file not found: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
        _ => Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("failed to read file: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        },
    })
}

/// Write to a file atomically (write to temp, then rename).
pub fn write_atomic(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or(Path::new("."));

    // Create parent directory if needed
    fs::create_dir_all(parent).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_WRITE_ERROR,
        message: format!("failed to create parent directory: {}", parent.display()),
        path: Some(parent.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })?;

    // Create temporary file in the same directory
    let mut temp_path = path.to_path_buf();
    if let Some(name) = path.file_name() {
        let temp_name = format!(".{}.tmp", name.to_string_lossy());
        temp_path.set_file_name(temp_name);
    } else {
        temp_path.push(".tmp");
    }

    // Write to temporary file
    {
        let mut file = File::create(&temp_path).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to create temporary file: {}", temp_path.display()),
            path: Some(temp_path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;

        file.write_all(contents).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to write to temporary file: {}", temp_path.display()),
            path: Some(temp_path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;

        file.sync_all().map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to sync temporary file: {}", temp_path.display()),
            path: Some(temp_path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;
    }

    // Atomic rename
    fs::rename(&temp_path, path).map_err(|e| {
        // Cleanup temp file on failure
        let _ = fs::remove_file(&temp_path);
        Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to rename temporary file to target: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        }
    })?;

    Ok(())
}

/// Write string to file atomically.
pub fn write_string_atomic(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    write_atomic(path, contents.as_bytes())
}

/// Ensure a directory exists (safe directory creation).
pub fn ensure_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to create directory: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;
    }
    Ok(())
}

/// Create a temporary file.
pub struct TempFile {
    path: PathBuf,
    file: Option<File>,
}

impl TempFile {
    /// Create a new temporary file.
    pub fn new() -> Result<Self> {
        Self::new_in(std::env::temp_dir())
    }

    /// Create a new temporary file in the specified directory.
    pub fn new_in(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        ensure_dir(dir)?;

        // Generate unique filename
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let filename = format!("tachikoma-{}.tmp", timestamp);
        let path = dir.join(filename);

        let file = File::create(&path).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to create temporary file: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;

        Ok(Self {
            path,
            file: Some(file),
        })
    }

    /// Get the path of the temporary file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get a mutable reference to the file.
    pub fn file(&mut self) -> Result<&mut File> {
        self.file.as_mut().ok_or_else(|| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: "temporary file has been consumed".to_string(),
            path: Some(self.path.to_string_lossy().to_string()),
            source: None,
        })
    }

    /// Write data to the temporary file.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.file()?.write_all(data).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to write to temporary file: {}", self.path.display()),
            path: Some(self.path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })
    }

    /// Consume the temporary file and persist it to the given path.
    pub fn persist(mut self, target: impl AsRef<Path>) -> Result<()> {
        let target = target.as_ref();
        if let Some(file) = self.file.take() {
            file.sync_all().map_err(|e| Error::FileSystem {
                code: ErrorCode::FILE_WRITE_ERROR,
                message: format!("failed to sync temporary file: {}", self.path.display()),
                path: Some(self.path.to_string_lossy().to_string()),
                source: Some(Box::new(e)),
            })?;
        }

        fs::rename(&self.path, target).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to persist temporary file to: {}", target.display()),
            path: Some(target.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;

        // Don't cleanup in drop since we've moved the file
        std::mem::forget(self);
        Ok(())
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // Clean up the temporary file
        let _ = fs::remove_file(&self.path);
    }
}

/// Set file permissions.
#[cfg(unix)]
pub fn set_permissions(path: impl AsRef<Path>, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    let path = path.as_ref();
    let permissions = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_WRITE_ERROR,
        message: format!("failed to set permissions for: {}", path.display()),
        path: Some(path.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })
}

/// Set file permissions (no-op on Windows).
#[cfg(not(unix))]
pub fn set_permissions(_path: impl AsRef<Path>, _mode: u32) -> Result<()> {
    // File permissions are not supported on Windows in the same way
    Ok(())
}

/// Make a file executable.
pub fn make_executable(path: impl AsRef<Path>) -> Result<()> {
    set_permissions(path, 0o755)
}

/// Make a file read-only.
pub fn make_readonly(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut permissions = fs::metadata(path)
        .map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("failed to read metadata: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?
        .permissions();
    
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_WRITE_ERROR,
        message: format!("failed to make file read-only: {}", path.display()),
        path: Some(path.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })
}

/// Check if a path exists and is a file.
pub fn is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

/// Check if a path exists and is a directory.
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// Get the file extension as a string.
pub fn extension(path: impl AsRef<Path>) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

/// Copy a file with optional overwrite.
pub fn copy_file(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    overwrite: bool,
) -> Result<u64> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    
    if !overwrite && dst.exists() {
        return Err(Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("destination already exists: {}", dst.display()),
            path: Some(dst.to_string_lossy().to_string()),
            source: None,
        });
    }

    if let Some(parent) = dst.parent() {
        ensure_dir(parent)?;
    }

    fs::copy(src, dst).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_WRITE_ERROR,
        message: format!("failed to copy {} to {}", src.display(), dst.display()),
        path: Some(dst.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })
}

/// Delete a file if it exists.
pub fn remove_file_if_exists(path: impl AsRef<Path>) -> Result<bool> {
    let path = path.as_ref();
    if path.exists() {
        fs::remove_file(path).map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_WRITE_ERROR,
            message: format!("failed to remove file: {}", path.display()),
            path: Some(path.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// List files in a directory.
pub fn list_files(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    let mut files = Vec::new();

    let read_dir = fs::read_dir(dir).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_READ_ERROR,
        message: format!("failed to read directory: {}", dir.display()),
        path: Some(dir.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("failed to read directory entry: {}", dir.display()),
            path: Some(dir.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;
        
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

/// List directories in a directory.
pub fn list_dirs(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    let mut dirs = Vec::new();

    let read_dir = fs::read_dir(dir).map_err(|e| Error::FileSystem {
        code: ErrorCode::FILE_READ_ERROR,
        message: format!("failed to read directory: {}", dir.display()),
        path: Some(dir.to_string_lossy().to_string()),
        source: Some(Box::new(e)),
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: format!("failed to read directory entry: {}", dir.display()),
            path: Some(dir.to_string_lossy().to_string()),
            source: Some(Box::new(e)),
        })?;
        
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }

    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_atomic_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        // Test initial write
        write_string_atomic(&path, "hello").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello");

        // Test overwrite
        write_string_atomic(&path, "world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "world");

        // Test binary write
        let binary_data = b"binary\x00data";
        write_atomic(&path, binary_data).unwrap();
        assert_eq!(fs::read(&path).unwrap(), binary_data);
    }

    #[test]
    fn test_ensure_dir() {
        let dir = tempdir().unwrap();
        let nested_path = dir.path().join("a/b/c");

        ensure_dir(&nested_path).unwrap();
        assert!(nested_path.is_dir());

        // Test calling again on existing directory
        ensure_dir(&nested_path).unwrap();
        assert!(nested_path.is_dir());
    }

    #[test]
    fn test_file_not_found() {
        let result = read_to_string("/nonexistent/path", 1024);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::FileSystem { code, .. } => {
                assert_eq!(code, ErrorCode::FILE_NOT_FOUND);
            }
            _ => panic!("Expected FileSystem error"),
        }
    }

    #[test]
    fn test_read_with_size_limit() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("large.txt");
        
        // Create a large file
        let large_content = "x".repeat(1000);
        fs::write(&path, &large_content).unwrap();

        // Test size limit
        let result = read_to_string(&path, 500);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("file too large"));

        // Test within limit
        let content = read_to_string(&path, 2000).unwrap();
        assert_eq!(content, large_content);
    }

    #[test]
    fn test_encoding_detection() {
        let dir = tempdir().unwrap();
        
        // Test UTF-8 with BOM
        let utf8_bom_path = dir.path().join("utf8_bom.txt");
        let utf8_content = "Hello, 世界!";
        let mut utf8_bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        utf8_bytes.extend_from_slice(utf8_content.as_bytes());
        fs::write(&utf8_bom_path, &utf8_bytes).unwrap();
        
        let detected = read_with_encoding_detection(&utf8_bom_path).unwrap();
        assert_eq!(detected, utf8_content);

        // Test plain UTF-8
        let utf8_path = dir.path().join("utf8.txt");
        fs::write(&utf8_path, utf8_content.as_bytes()).unwrap();
        
        let detected = read_with_encoding_detection(&utf8_path).unwrap();
        assert_eq!(detected, utf8_content);

        // Test non-UTF-8 (latin-1) - use simple ASCII that will be the same in both
        let latin1_path = dir.path().join("latin1.txt");
        let latin1_bytes = vec![0x41, 0x42, 0x43, 0x44, 0x45]; // ABCDE in latin-1
        fs::write(&latin1_path, &latin1_bytes).unwrap();
        
        let detected = read_with_encoding_detection(&latin1_path).unwrap();
        assert_eq!(detected, "ABCDE"); // Should decode correctly
        assert_eq!(detected.len(), 5); // Should decode as 5 characters
    }

    #[test]
    fn test_temp_file() {
        let mut temp = TempFile::new().unwrap();
        
        // Test writing to temp file
        temp.write(b"temporary data").unwrap();
        
        // Test that file exists
        assert!(temp.path().exists());
        
        let target_dir = tempdir().unwrap();
        let target_path = target_dir.path().join("persisted.txt");
        
        // Test persist
        temp.persist(&target_path).unwrap();
        
        // Verify persisted file
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "temporary data");
    }

    #[test]
    fn test_temp_file_cleanup() {
        let temp_path = {
            let temp = TempFile::new().unwrap();
            let path = temp.path().to_path_buf();
            assert!(path.exists());
            path
        };
        
        // After temp goes out of scope, file should be cleaned up
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_copy_file() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("source.txt");
        let dst = dir.path().join("dest.txt");
        
        fs::write(&src, "test content").unwrap();
        
        // Test copy
        let bytes_copied = copy_file(&src, &dst, false).unwrap();
        assert!(bytes_copied > 0);
        assert_eq!(fs::read_to_string(&dst).unwrap(), "test content");
        
        // Test overwrite protection
        let result = copy_file(&src, &dst, false);
        assert!(result.is_err());
        
        // Test overwrite allowed
        fs::write(&src, "new content").unwrap();
        copy_file(&src, &dst, true).unwrap();
        assert_eq!(fs::read_to_string(&dst).unwrap(), "new content");
    }

    #[test]
    fn test_remove_file_if_exists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        
        // Test on non-existent file
        let removed = remove_file_if_exists(&path).unwrap();
        assert!(!removed);
        
        // Test on existing file
        fs::write(&path, "test").unwrap();
        let removed = remove_file_if_exists(&path).unwrap();
        assert!(removed);
        assert!(!path.exists());
    }

    #[test]
    fn test_list_files_and_dirs() {
        let dir = tempdir().unwrap();
        
        // Create test structure
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("file1.txt"), "content").unwrap();
        fs::write(dir.path().join("file2.txt"), "content").unwrap();
        fs::write(dir.path().join("subdir/file3.txt"), "content").unwrap();
        
        // Test list files
        let files = list_files(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.txt"));
        
        // Test list dirs
        let dirs = list_dirs(dir.path()).unwrap();
        assert_eq!(dirs.len(), 1);
        assert!(dirs.iter().any(|d| d.file_name().unwrap() == "subdir"));
    }

    #[test]
    fn test_file_type_checks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let dir_path = dir.path().join("subdir");
        
        fs::write(&file_path, "test").unwrap();
        fs::create_dir(&dir_path).unwrap();
        
        assert!(is_file(&file_path));
        assert!(!is_file(&dir_path));
        assert!(!is_dir(&file_path));
        assert!(is_dir(&dir_path));
        
        assert!(!is_file(&dir.path().join("nonexistent")));
        assert!(!is_dir(&dir.path().join("nonexistent")));
    }

    #[test]
    fn test_extension() {
        assert_eq!(extension("test.txt"), Some("txt".to_string()));
        assert_eq!(extension("test.TAR.GZ"), Some("gz".to_string()));
        assert_eq!(extension("test"), None);
        assert_eq!(extension(".hidden"), None);
        assert_eq!(extension("path/test.json"), Some("json".to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn test_permissions() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        
        fs::write(&path, "test").unwrap();
        
        // Test set permissions
        set_permissions(&path, 0o644).unwrap();
        make_readonly(&path).unwrap();
        make_executable(&path).unwrap();
        
        // Verify file still exists and is readable
        assert!(path.exists());
        assert!(fs::read_to_string(&path).is_ok());
    }
}