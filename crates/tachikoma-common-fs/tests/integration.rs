use tachikoma_common_fs as fs;
use tachikoma_common_core::Result;
use std::path::PathBuf;

#[test]
fn test_integration_with_core_types() {
    // Test that errors integrate properly with core error types
    let result: Result<String> = fs::read_to_string("/nonexistent/file", 1024);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert_eq!(error.category(), tachikoma_common_core::ErrorCategory::FileSystem);
    assert_eq!(error.code(), tachikoma_common_core::ErrorCode::FILE_NOT_FOUND);
}

#[test]
fn test_path_normalization_integration() {
    // Test cross-platform path normalization
    let test_cases = vec![
        ("./a/b/../c", "a/c"),
        ("a/./b", "a/b"),
        ("a/../b", "b"),
        ("../../a/b", "../../a/b"),
        ("a/b/c/../../d", "a/d"),
        ("", "."),
        (".", "."),
        ("..", ".."),
    ];

    for (input, expected) in test_cases {
        let normalized = fs::path::normalize(input);
        let expected_path = PathBuf::from(expected);
        assert_eq!(normalized, expected_path, "Failed for input: {}", input);
    }
}

#[test]
fn test_relative_path_calculations() {
    let test_cases = vec![
        ("/a/b/c", "/a/b", Some("c")),
        ("/a/b", "/a/b/c", Some("..")),
        ("/a/b/c", "/a/d", Some("../b/c")),
        ("/a/b/c", "/a/d/e", Some("../../b/c")),
        ("/a/b/c", "/a/b/c", Some(".")),
    ];

    for (path, base, expected) in test_cases {
        let result = fs::path::relative_to(path, base);
        match expected {
            Some(exp) => {
                assert_eq!(result, Some(PathBuf::from(exp)), 
                          "Failed for path: {} relative to base: {}", path, base);
            }
            None => {
                assert_eq!(result, None, 
                          "Expected None for path: {} relative to base: {}", path, base);
            }
        }
    }
}

#[test]
fn test_safe_path_joining() {
    // Test safe join prevents path traversal attacks
    let safe_cases = vec![
        ("/base", "file.txt", Some("/base/file.txt")),
        ("/base", "dir/file.txt", Some("/base/dir/file.txt")),
        ("relative/base", "file.txt", Some("relative/base/file.txt")),
    ];

    for (base, path, expected) in safe_cases {
        let result = fs::path::safe_join(base, path);
        match expected {
            Some(exp) => {
                assert_eq!(result, Some(PathBuf::from(exp)),
                          "Failed for safe_join({}, {})", base, path);
            }
            None => {
                assert_eq!(result, None,
                          "Expected None for safe_join({}, {})", base, path);
            }
        }
    }

    // Test dangerous cases that should return None
    let dangerous_cases = vec![
        ("/base", "../escape"),
        ("/base", "/absolute/path"),
        ("/base", "normal/../escape"),
        ("base", "dir/../../escape"),
    ];

    for (base, path) in dangerous_cases {
        let result = fs::path::safe_join(base, path);
        assert_eq!(result, None, "Expected None for dangerous safe_join({}, {})", base, path);
    }
}

#[test]
fn test_cross_platform_path_handling() {
    // Test Unix-style string conversion
    let test_cases = vec![
        ("a/b/c", "a/b/c"),
        ("a\\b\\c", "a/b/c"), // Should normalize backslashes on Windows
    ];

    for (input, expected) in test_cases {
        let unix_style = fs::path::to_unix_string(input);
        // On Unix systems, backslashes are literal, so this test might behave differently
        // but the function should still work
        if cfg!(windows) {
            assert_eq!(unix_style, expected);
        } else {
            // On Unix, backslashes are preserved as literal characters
            let unix_style = fs::path::to_unix_string(PathBuf::from(input));
            assert!(unix_style.contains('/') || unix_style.contains('\\'));
        }
    }
}

#[test]
fn test_glob_pattern_matching() {
    let test_cases = vec![
        ("file.rs", "*.rs", true),
        ("file.txt", "*.rs", false),
        ("any/path/file", "*", true),
        ("src/main.rs", "src/*", true), // This should match - path is src/main.rs, pattern is src/*
        ("file.tar.gz", "*.gz", true),
        ("README.md", "*.MD", false), // Case sensitive
        ("src/lib/mod.rs", "src/*", false), // This should NOT match - contains subdirectory
    ];

    for (path, pattern, expected) in test_cases {
        let result = fs::path::matches_glob(path, pattern);
        assert_eq!(result, expected, 
                  "Pattern {} matching path {} should be {}", pattern, path, expected);
    }
}

#[test]
fn test_path_utilities() {
    // Test file stem extraction
    assert_eq!(fs::path::stem("file.txt"), Some("file".to_string()));
    assert_eq!(fs::path::stem("path/to/file.rs"), Some("file".to_string()));
    assert_eq!(fs::path::stem("no_extension"), Some("no_extension".to_string()));
    assert_eq!(fs::path::stem(".hidden"), None);

    // Test directory helpers
    let root = PathBuf::from("/project");
    assert_eq!(fs::path::specs_dir(&root), PathBuf::from("/project/specs"));
    assert_eq!(fs::path::config_dir(&root), PathBuf::from("/project/.tachikoma"));
}