//! Path manipulation utilities.

use std::path::{Component, Path, PathBuf};

/// Normalize a path by resolving `.` and `..` without hitting the filesystem.
pub fn normalize(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::Prefix(p) => components.push(Component::Prefix(p)),
            Component::RootDir => {
                components.clear();
                components.push(Component::RootDir);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = components.last() {
                    match last {
                        Component::Normal(_) => {
                            components.pop();
                        }
                        Component::ParentDir => {
                            components.push(Component::ParentDir);
                        }
                        _ => {} // Don't pop prefix or root dir
                    }
                } else {
                    components.push(Component::ParentDir);
                }
            }
            Component::Normal(c) => components.push(Component::Normal(c)),
        }
    }

    if components.is_empty() {
        PathBuf::from(".")
    } else {
        components.iter().collect()
    }
}

/// Make a path relative to a base path.
pub fn relative_to(path: impl AsRef<Path>, base: impl AsRef<Path>) -> Option<PathBuf> {
    let path = normalize(path);
    let base = normalize(base);

    let mut path_components = path.components().peekable();
    let mut base_components = base.components().peekable();

    // Skip common prefix
    while let (Some(p), Some(b)) = (path_components.peek(), base_components.peek()) {
        if p != b {
            break;
        }
        path_components.next();
        base_components.next();
    }

    // Count remaining base components (need `..` for each)
    let mut result = PathBuf::new();
    for _ in base_components {
        result.push("..");
    }

    // Add remaining path components
    for component in path_components {
        result.push(component);
    }

    if result.as_os_str().is_empty() {
        Some(PathBuf::from("."))
    } else {
        Some(result)
    }
}

/// Join paths safely, preventing path traversal attacks.
pub fn safe_join(base: impl AsRef<Path>, path: impl AsRef<Path>) -> Option<PathBuf> {
    let base = base.as_ref();
    let path = path.as_ref();

    // Check for absolute paths or path traversal
    if path.is_absolute() {
        return None;
    }

    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return None;
        }
    }

    Some(base.join(path))
}

/// Find the project root by looking for marker files.
pub fn find_project_root(start: impl AsRef<Path>) -> Option<PathBuf> {
    let markers = [
        ".tachikoma",
        "Cargo.toml",
        "package.json",
        ".git",
    ];

    let mut current = start.as_ref().to_path_buf();

    loop {
        for marker in &markers {
            if current.join(marker).exists() {
                return Some(current);
            }
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Get the specs directory for a project.
pub fn specs_dir(project_root: impl AsRef<Path>) -> PathBuf {
    project_root.as_ref().join("specs")
}

/// Get the .tachikoma config directory.
pub fn config_dir(project_root: impl AsRef<Path>) -> PathBuf {
    project_root.as_ref().join(".tachikoma")
}

/// Convert a path to a Unix-style string (forward slashes).
pub fn to_unix_string(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Get the file stem (name without extension).
pub fn stem(path: impl AsRef<Path>) -> Option<String> {
    let path = path.as_ref();
    let file_name = path.file_name()?.to_str()?;
    
    // Handle hidden files (starting with .)
    if file_name.starts_with('.') && file_name.len() > 1 && !file_name[1..].contains('.') {
        // Hidden file with no extension
        return None;
    }
    
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Check if a path matches a glob pattern.
pub fn matches_glob(path: impl AsRef<Path>, pattern: &str) -> bool {
    let path_str = path.as_ref().to_string_lossy();

    // Simple glob matching (just * for now)
    if pattern == "*" {
        return true;
    }

    if let Some(suffix) = pattern.strip_prefix("*.") {
        return path_str.ends_with(&format!(".{}", suffix));
    }

    if let Some(prefix) = pattern.strip_suffix("/*") {
        // Check if path starts with prefix followed by exactly one more segment
        if path_str.starts_with(prefix) && path_str.len() > prefix.len() {
            let remainder = &path_str[prefix.len()..];
            // Should start with / and not contain any more /
            return remainder.starts_with('/') && !remainder[1..].contains('/');
        }
        return false;
    }

    path_str == pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("a/b/../c"), PathBuf::from("a/c"));
        assert_eq!(normalize("a/./b"), PathBuf::from("a/b"));
        assert_eq!(normalize("../a/b"), PathBuf::from("../a/b"));
        assert_eq!(normalize("/a/../b"), PathBuf::from("/b"));
    }

    #[test]
    fn test_relative_to() {
        assert_eq!(
            relative_to("/a/b/c", "/a/b"),
            Some(PathBuf::from("c"))
        );
        assert_eq!(
            relative_to("/a/b", "/a/b/c"),
            Some(PathBuf::from(".."))
        );
        assert_eq!(
            relative_to("/a/b/c", "/a/d/e"),
            Some(PathBuf::from("../../b/c"))
        );
    }

    #[test]
    fn test_safe_join() {
        assert_eq!(
            safe_join("/base", "file.txt"),
            Some(PathBuf::from("/base/file.txt"))
        );
        assert_eq!(safe_join("/base", "../escape"), None);
        assert_eq!(safe_join("/base", "/absolute"), None);
    }

    #[test]
    fn test_find_project_root() {
        // This should find the project root since we have Cargo.toml
        let current_dir = std::env::current_dir().unwrap();
        let root = find_project_root(&current_dir);
        assert!(root.is_some());
        
        if let Some(root_path) = root {
            // Should find Cargo.toml or .git in the root
            let has_cargo = root_path.join("Cargo.toml").exists();
            let has_git = root_path.join(".git").exists();
            let has_tachikoma = root_path.join(".tachikoma").exists();
            
            assert!(has_cargo || has_git || has_tachikoma);
        }
    }
}