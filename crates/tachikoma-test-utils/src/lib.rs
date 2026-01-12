//! Test utilities for Tachikoma crates.

use std::path::PathBuf;
use tempfile::TempDir;

/// Creates a temporary directory that is cleaned up on drop.
pub fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Creates a temporary file with given content.
pub fn temp_file(content: &str) -> (TempDir, PathBuf) {
    let dir = temp_dir();
    let path = dir.path().join("test_file");
    std::fs::write(&path, content).expect("Failed to write temp file");
    (dir, path)
}

/// Macro for async tests with tokio runtime.
#[macro_export]
macro_rules! async_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            $body
        }
    };
}

/// Assert that a Result is Ok and return the value.
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a Result is Err.
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(v) => panic!("Expected Err, got Ok: {:?}", v),
            Err(_) => {}
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_temp_dir_creation() {
        let dir = temp_dir();
        assert!(dir.path().exists());
        assert!(dir.path().is_dir());
    }

    #[test]
    fn test_temp_file_creation() {
        let content = "test content";
        let (_dir, path) = temp_file(content);
        assert!(path.exists());
        assert!(path.is_file());
        let read_content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(read_content, content);
    }

    // Example property test
    proptest! {
        #[test]
        fn test_temp_file_content_roundtrip(content in "\\PC*") {
            let (_dir, path) = temp_file(&content);
            let read_content = std::fs::read_to_string(&path).unwrap();
            prop_assert_eq!(content, read_content);
        }

        #[test]
        fn test_path_normalization(
            parts in prop::collection::vec("[a-z]+", 1..5)
        ) {
            let path_str = parts.join("/");
            let path = std::path::Path::new(&path_str);
            
            // Property: all path components are present
            let components: Vec<_> = path.components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();
            
            for part in &parts {
                prop_assert!(components.contains(&part.as_str()));
            }
        }
    }
}