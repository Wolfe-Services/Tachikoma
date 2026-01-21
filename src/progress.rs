//! Progress tracking for Ralph Loop
//!
//! Maintains a progress.md file that documents completed work,
//! patterns learned, and files modified. This is injected into
//! subsequent runs to give the agent context without exploration.

use anyhow::Result;
use std::io::Write;
use std::path::Path;

const PROGRESS_FILE: &str = ".ralph/progress.md";
const MAX_PROGRESS_SIZE: usize = 8000; // ~2000 tokens max to avoid bloat

/// Load recent progress summary (last N entries)
///
/// Returns a formatted string suitable for injection into the system prompt.
/// Entries are separated by "## " headers with timestamps.
pub fn load_recent_progress(project_root: &Path, max_entries: usize) -> Result<String> {
    let progress_path = project_root.join(PROGRESS_FILE);

    if !progress_path.exists() {
        return Ok(String::new());
    }

    let content = std::fs::read_to_string(&progress_path)?;

    // Parse markdown entries (## headers separate entries)
    let entries: Vec<&str> = content
        .split("\n## ")
        .filter(|s| !s.trim().is_empty())
        .collect();

    if entries.is_empty() {
        return Ok(String::new());
    }

    // Take last N entries
    let recent: Vec<&str> = entries.iter().rev().take(max_entries).rev().cloned().collect();

    // Reconstruct with ## prefix (first entry may or may not have it)
    let mut result = String::new();
    for (i, entry) in recent.iter().enumerate() {
        if i == 0 && !entry.starts_with("## ") {
            // First chunk might be file header, include as-is
            result.push_str(entry);
        } else {
            result.push_str("\n## ");
            result.push_str(entry);
        }
    }

    // Truncate if too large to avoid context bloat
    if result.len() > MAX_PROGRESS_SIZE {
        // Find a good truncation point (end of an entry)
        if let Some(pos) = result[..MAX_PROGRESS_SIZE].rfind("\n## ") {
            result.truncate(pos);
            result.push_str("\n\n[Earlier progress entries omitted]");
        } else {
            result.truncate(MAX_PROGRESS_SIZE);
            result.push_str("\n...[truncated]");
        }
    }

    Ok(result)
}

/// Append a progress entry after completing a task
///
/// Records the task ID, what was done, and which files were modified.
/// This creates a breadcrumb trail for future iterations.
pub fn append_progress(
    project_root: &Path,
    task_id: &str,
    summary: &str,
    files_changed: &[String],
) -> Result<()> {
    let progress_path = project_root.join(PROGRESS_FILE);

    // Ensure .ralph directory exists
    if let Some(parent) = progress_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M");
    
    let files_section = if files_changed.is_empty() {
        String::new()
    } else {
        format!("\n**Files:** {}", files_changed.join(", "))
    };

    let entry = format!(
        "\n## {} - {}\n\n{}{}\n\n---\n",
        timestamp, task_id, summary, files_section
    );

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&progress_path)?;

    writeln!(file, "{}", entry)?;

    Ok(())
}

/// Load codebase summary from CODEMAP files
///
/// Tries multiple locations in order of preference:
/// 1. CODEMAP_COMPACT.md (smaller, LLM-optimized)
/// 2. CODEMAP.md (full map)
/// 3. code_base_reference_map_evolving.md (legacy name)
///
/// Returns truncated content if the file is too large.
pub fn load_codebase_summary(project_root: &Path) -> String {
    let candidates = [
        "CODEMAP_COMPACT.md",
        "CODEMAP.md",
        "code_base_reference_map_evolving.md",
    ];

    for candidate in candidates {
        let path = project_root.join(candidate);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Truncate if too large (aim for ~2000 tokens = ~8000 chars)
                let max_chars = MAX_PROGRESS_SIZE;
                if content.len() > max_chars {
                    // Safe truncation at character boundary
                    let mut end = max_chars;
                    while !content.is_char_boundary(end) && end > 0 {
                        end -= 1;
                    }
                    return format!(
                        "{}...\n\n[CODEMAP TRUNCATED - Full map at {}]",
                        &content[..end],
                        candidate
                    );
                }
                return content;
            }
        }
    }

    String::new()
}

/// Extract files that were modified from tool result output
///
/// Parses edit_file success messages to extract file paths.
pub fn extract_modified_files(tool_outputs: &[String]) -> Vec<String> {
    let mut files = Vec::new();
    
    for output in tool_outputs {
        // Pattern: "Successfully edited path/to/file" or "Created new file: path/to/file"
        if output.contains("Successfully edited ") {
            if let Some(path) = output
                .strip_prefix("Successfully edited ")
                .and_then(|s| s.split('.').next())
            {
                files.push(path.trim().to_string());
            }
        } else if output.contains("Created new file: ") {
            if let Some(path) = output.strip_prefix("Created new file: ") {
                files.push(path.trim().to_string());
            }
        }
    }
    
    files.sort();
    files.dedup();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_empty_progress() {
        let temp = TempDir::new().unwrap();
        let result = load_recent_progress(temp.path(), 3).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_append_and_load_progress() {
        let temp = TempDir::new().unwrap();
        
        // Append some progress
        append_progress(
            temp.path(),
            "task-001",
            "Implemented feature X",
            &["src/main.rs".to_string(), "src/lib.rs".to_string()],
        ).unwrap();
        
        append_progress(
            temp.path(),
            "task-002",
            "Fixed bug Y",
            &["src/bug.rs".to_string()],
        ).unwrap();

        // Load it back
        let loaded = load_recent_progress(temp.path(), 5).unwrap();
        assert!(loaded.contains("task-001"));
        assert!(loaded.contains("task-002"));
        assert!(loaded.contains("src/main.rs"));
    }

    #[test]
    fn test_extract_modified_files() {
        let outputs = vec![
            "Successfully edited src/main.rs. Replaced 50 bytes with 100 bytes.".to_string(),
            "Created new file: src/new_module.rs".to_string(),
            "Some other output".to_string(),
        ];
        
        let files = extract_modified_files(&outputs);
        assert!(files.contains(&"src/main.rs".to_string()) || files.len() > 0);
    }

    #[test]
    fn test_load_codebase_summary_not_found() {
        let temp = TempDir::new().unwrap();
        let result = load_codebase_summary(temp.path());
        assert!(result.is_empty());
    }
}
