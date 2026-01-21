//! The Six Primitives - Minimum viable toolbelt for agentic coding
//!
//! 1. read_file - Read file contents
//! 2. list_files - List directory contents
//! 3. bash - Execute shell commands with timeout
//! 4. edit_file - Modify files with uniqueness check
//! 5. code_search - Ripgrep wrapper for pattern search
//! 6. beads - Issue tracker operations (show, update, close, ready)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

/// Tool definition for Claude API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
        }
    }
}

/// Get all tool definitions for Claude API
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file at the given path. Returns the file contents as a string.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List files and directories at the given path. Returns a list of entries with their types (file/directory).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The directory path to list"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to list recursively (default: false)"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "bash".to_string(),
            description: "Execute a bash command. Returns stdout, stderr, and exit code. Has a timeout of 120 seconds by default.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    },
                    "timeout_secs": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 120, max: 600)"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory for the command"
                    }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: "edit_file".to_string(),
            description: "Edit a file by replacing old_string with new_string. The old_string must be unique in the file (appear exactly once). For creating new files, use old_string as empty string.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to edit"
                    },
                    "old_string": {
                        "type": "string",
                        "description": "The exact string to find and replace (must be unique). Empty string for new file."
                    },
                    "new_string": {
                        "type": "string",
                        "description": "The string to replace old_string with"
                    }
                },
                "required": ["path", "old_string", "new_string"]
            }),
        },
        ToolDefinition {
            name: "code_search".to_string(),
            description: "Search for a pattern in the codebase using ripgrep. Returns matching lines with file paths and line numbers.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "The directory to search in (default: current directory)"
                    },
                    "file_pattern": {
                        "type": "string",
                        "description": "Glob pattern to filter files (e.g., '*.rs', '*.ts')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 50)"
                    }
                },
                "required": ["pattern"]
            }),
        },
        ToolDefinition {
            name: "beads".to_string(),
            description: "Interact with the beads issue tracker. Actions: 'ready' (list unblocked tasks), 'show' (get task details), 'update' (change task status), 'close' (mark task complete), 'sync' (commit beads changes).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["ready", "show", "update", "close", "sync"],
                        "description": "The beads action to perform"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID (required for show, update, close)"
                    },
                    "status": {
                        "type": "string",
                        "enum": ["open", "in_progress", "closed"],
                        "description": "New status (for update action)"
                    },
                    "reason": {
                        "type": "string",
                        "description": "Reason for closing (for close action)"
                    }
                },
                "required": ["action"]
            }),
        },
    ]
}

/// Execute a tool call
pub async fn execute_tool(name: &str, input: &serde_json::Value, project_root: &Path) -> ToolResult {
    match name {
        "read_file" => read_file(input, project_root).await,
        "list_files" => list_files(input, project_root).await,
        "bash" => bash(input, project_root).await,
        "edit_file" => edit_file(input, project_root).await,
        "code_search" => code_search(input, project_root).await,
        "beads" => beads(input, project_root).await,
        _ => ToolResult::error(format!("Unknown tool: {}", name)),
    }
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// 1. read_file - Read file contents
async fn read_file(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let path_str = match input.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter: path"),
    };

    let path = resolve_path(path_str, project_root);

    match tokio::fs::read_to_string(&path).await {
        Ok(content) => {
            // Truncate if too large (avoid blowing up context)
            let max_size = 100_000; // ~100KB
            if content.len() > max_size {
                let truncated = &content[..max_size];
                ToolResult::success(format!(
                    "{}\n\n[Truncated: file is {} bytes, showing first {} bytes]",
                    truncated,
                    content.len(),
                    max_size
                ))
            } else {
                ToolResult::success(content)
            }
        }
        Err(e) => ToolResult::error(format!("Failed to read file {}: {}", path.display(), e)),
    }
}

/// 2. list_files - List directory contents
async fn list_files(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let path_str = match input.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter: path"),
    };

    let recursive = input.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
    let path = resolve_path(path_str, project_root);

    if recursive {
        list_files_recursive(&path).await
    } else {
        list_files_flat(&path).await
    }
}

async fn list_files_flat(path: &Path) -> ToolResult {
    let mut entries = Vec::new();

    let mut dir = match tokio::fs::read_dir(path).await {
        Ok(d) => d,
        Err(e) => return ToolResult::error(format!("Failed to read directory: {}", e)),
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let file_type = match entry.file_type().await {
            Ok(ft) => {
                if ft.is_dir() {
                    "dir"
                } else if ft.is_symlink() {
                    "symlink"
                } else {
                    "file"
                }
            }
            Err(_) => "unknown",
        };

        entries.push(format!("{}\t{}", file_type, entry.file_name().to_string_lossy()));
    }

    entries.sort();
    ToolResult::success(entries.join("\n"))
}

async fn list_files_recursive(path: &Path) -> ToolResult {
    let mut entries = Vec::new();
    let mut stack = vec![path.to_path_buf()];

    while let Some(current) = stack.pop() {
        let mut dir = match tokio::fs::read_dir(&current).await {
            Ok(d) => d,
            Err(_) => continue,
        };

        while let Ok(Some(entry)) = dir.next_entry().await {
            let entry_path = entry.path();
            let relative = entry_path.strip_prefix(path).unwrap_or(&entry_path);

            if let Ok(ft) = entry.file_type().await {
                if ft.is_dir() {
                    // Skip common ignored directories
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if !["node_modules", ".git", "target", ".next", "dist", "__pycache__"]
                        .contains(&name_str.as_ref())
                    {
                        stack.push(entry_path.clone());
                        entries.push(format!("dir\t{}/", relative.display()));
                    }
                } else {
                    entries.push(format!("file\t{}", relative.display()));
                }
            }
        }

        // Limit to avoid blowing up context
        if entries.len() > 1000 {
            entries.push("[Truncated: too many entries]".to_string());
            break;
        }
    }

    entries.sort();
    ToolResult::success(entries.join("\n"))
}

/// 3. bash - Execute shell commands with timeout
async fn bash(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let command = match input.get("command").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return ToolResult::error("Missing required parameter: command"),
    };

    let timeout_secs = input
        .get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(120)
        .min(600); // Max 10 minutes

    let cwd = input
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(|p| resolve_path(p, project_root))
        .unwrap_or_else(|| project_root.to_path_buf());

    let child = match Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(&cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return ToolResult::error(format!("Failed to spawn command: {}", e)),
    };

    // Get PID before consuming child (for potential kill on timeout)
    let pid = child.id();

    let result = timeout(Duration::from_secs(timeout_secs), child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let exit_code = output.status.code().unwrap_or(-1);

            let combined = format!(
                "Exit code: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
                exit_code, stdout, stderr
            );

            // Truncate if too large
            let max_size = 50_000;
            if combined.len() > max_size {
                ToolResult::success(format!(
                    "{}\n\n[Truncated: output is {} bytes]",
                    &combined[..max_size],
                    combined.len()
                ))
            } else {
                ToolResult::success(combined)
            }
        }
        Ok(Err(e)) => ToolResult::error(format!("Command failed: {}", e)),
        Err(_) => {
            // Timeout - try to kill the process by PID
            if let Some(pid) = pid {
                // Best effort kill - process may have already exited
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
            }
            ToolResult::error(format!("Command timed out after {} seconds", timeout_secs))
        }
    }
}

/// 4. edit_file - Modify files with uniqueness check
async fn edit_file(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let path_str = match input.get("path").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter: path"),
    };

    let old_string = match input.get("old_string").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return ToolResult::error("Missing required parameter: old_string"),
    };

    let new_string = match input.get("new_string").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return ToolResult::error("Missing required parameter: new_string"),
    };

    let path = resolve_path(path_str, project_root);

    // Handle new file creation
    if old_string.is_empty() {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return ToolResult::error(format!("Failed to create directory: {}", e));
            }
        }

        return match tokio::fs::write(&path, new_string).await {
            Ok(_) => ToolResult::success(format!("Created new file: {}", path.display())),
            Err(e) => ToolResult::error(format!("Failed to create file: {}", e)),
        };
    }

    // Read existing file
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(e) => return ToolResult::error(format!("Failed to read file: {}", e)),
    };

    // Check uniqueness
    let matches: Vec<_> = content.match_indices(old_string).collect();

    if matches.is_empty() {
        return ToolResult::error(format!(
            "old_string not found in file. Make sure it matches exactly (including whitespace).\n\nSearched for:\n{}\n\nIn file: {}",
            old_string,
            path.display()
        ));
    }

    if matches.len() > 1 {
        return ToolResult::error(format!(
            "old_string is not unique - found {} matches. Add more context to make it unique.",
            matches.len()
        ));
    }

    // Perform replacement
    let new_content = content.replacen(old_string, new_string, 1);

    match tokio::fs::write(&path, new_content).await {
        Ok(_) => ToolResult::success(format!(
            "Successfully edited {}. Replaced {} bytes with {} bytes.",
            path.display(),
            old_string.len(),
            new_string.len()
        )),
        Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
    }
}

/// 5. code_search - Ripgrep wrapper
async fn code_search(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let pattern = match input.get("pattern").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return ToolResult::error("Missing required parameter: pattern"),
    };

    let search_path = input
        .get("path")
        .and_then(|v| v.as_str())
        .map(|p| resolve_path(p, project_root))
        .unwrap_or_else(|| project_root.to_path_buf());

    let file_pattern = input.get("file_pattern").and_then(|v| v.as_str());
    let max_results = input
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;

    let mut cmd = Command::new("rg");
    cmd.arg("--json")
        .arg("--max-count")
        .arg(max_results.to_string())
        .arg("-n") // line numbers
        .arg("--no-heading");

    if let Some(fp) = file_pattern {
        cmd.arg("-g").arg(fp);
    }

    // Ignore common directories
    cmd.arg("--glob").arg("!node_modules")
        .arg("--glob").arg("!.git")
        .arg("--glob").arg("!target")
        .arg("--glob").arg("!dist");

    cmd.arg(pattern).arg(&search_path);

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(e) => {
            return ToolResult::error(format!(
                "Failed to run ripgrep (is rg installed?): {}",
                e
            ))
        }
    };

    if !output.status.success() && output.stdout.is_empty() {
        // No matches found is not an error
        if output.status.code() == Some(1) {
            return ToolResult::success("No matches found.");
        }
        return ToolResult::error(format!(
            "ripgrep failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse JSON output into human-readable format
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if json.get("type").and_then(|v| v.as_str()) == Some("match") {
                if let Some(data) = json.get("data") {
                    let path = data.get("path").and_then(|p| p.get("text")).and_then(|t| t.as_str()).unwrap_or("");
                    let line_num = data.get("line_number").and_then(|l| l.as_u64()).unwrap_or(0);
                    let text = data.get("lines").and_then(|l| l.get("text")).and_then(|t| t.as_str()).unwrap_or("");

                    results.push(format!("{}:{}: {}", path, line_num, text.trim()));
                }
            }
        }
    }

    if results.is_empty() {
        ToolResult::success("No matches found.")
    } else {
        ToolResult::success(format!("Found {} matches:\n\n{}", results.len(), results.join("\n")))
    }
}

/// 6. beads - Issue tracker operations
async fn beads(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let action = match input.get("action").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return ToolResult::error("Missing required parameter: action"),
    };

    match action {
        "ready" => beads_ready(project_root).await,
        "show" => {
            let task_id = match input.get("task_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter: task_id for show action"),
            };
            beads_show(project_root, task_id).await
        }
        "update" => {
            let task_id = match input.get("task_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter: task_id for update action"),
            };
            let status = match input.get("status").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolResult::error("Missing required parameter: status for update action"),
            };
            beads_update(project_root, task_id, status).await
        }
        "close" => {
            let task_id = match input.get("task_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return ToolResult::error("Missing required parameter: task_id for close action"),
            };
            let reason = input.get("reason").and_then(|v| v.as_str());
            beads_close(project_root, task_id, reason).await
        }
        "sync" => beads_sync(project_root).await,
        _ => ToolResult::error(format!("Unknown beads action: {}", action)),
    }
}

async fn beads_ready(project_root: &Path) -> ToolResult {
    let output = Command::new("bd")
        .args(["ready"])
        .current_dir(project_root)
        .output()
        .await;

    match output {
        Ok(out) => {
            if out.status.success() {
                ToolResult::success(String::from_utf8_lossy(&out.stdout).to_string())
            } else {
                ToolResult::error(format!(
                    "bd ready failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ))
            }
        }
        Err(e) => ToolResult::error(format!("Failed to run bd ready: {}", e)),
    }
}

async fn beads_show(project_root: &Path, task_id: &str) -> ToolResult {
    let output = Command::new("bd")
        .args(["show", task_id])
        .current_dir(project_root)
        .output()
        .await;

    match output {
        Ok(out) => {
            if out.status.success() {
                ToolResult::success(String::from_utf8_lossy(&out.stdout).to_string())
            } else {
                ToolResult::error(format!(
                    "bd show {} failed: {}",
                    task_id,
                    String::from_utf8_lossy(&out.stderr)
                ))
            }
        }
        Err(e) => ToolResult::error(format!("Failed to run bd show: {}", e)),
    }
}

async fn beads_update(project_root: &Path, task_id: &str, status: &str) -> ToolResult {
    let output = Command::new("bd")
        .args(["update", task_id, &format!("--status={}", status)])
        .current_dir(project_root)
        .output()
        .await;

    match output {
        Ok(out) => {
            if out.status.success() {
                ToolResult::success(format!("Updated {} status to {}", task_id, status))
            } else {
                ToolResult::error(format!(
                    "bd update failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ))
            }
        }
        Err(e) => ToolResult::error(format!("Failed to run bd update: {}", e)),
    }
}

async fn beads_close(project_root: &Path, task_id: &str, reason: Option<&str>) -> ToolResult {
    let mut args = vec!["close", task_id];
    let reason_arg;
    
    if let Some(r) = reason {
        reason_arg = format!("--reason={}", r);
        args.push(&reason_arg);
    }

    let output = Command::new("bd")
        .args(&args)
        .current_dir(project_root)
        .output()
        .await;

    match output {
        Ok(out) => {
            if out.status.success() {
                ToolResult::success(format!("Closed task {}", task_id))
            } else {
                ToolResult::error(format!(
                    "bd close failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ))
            }
        }
        Err(e) => ToolResult::error(format!("Failed to run bd close: {}", e)),
    }
}

async fn beads_sync(project_root: &Path) -> ToolResult {
    let output = Command::new("bd")
        .args(["sync"])
        .current_dir(project_root)
        .output()
        .await;

    match output {
        Ok(out) => {
            if out.status.success() {
                ToolResult::success("Synced beads state")
            } else {
                // Sync warnings aren't critical
                ToolResult::success(format!(
                    "Synced (with warnings): {}",
                    String::from_utf8_lossy(&out.stderr)
                ))
            }
        }
        Err(e) => ToolResult::error(format!("Failed to run bd sync: {}", e)),
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Resolve a path relative to project root
fn resolve_path(path_str: &str, project_root: &Path) -> std::path::PathBuf {
    let path = Path::new(path_str);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        let tools = get_tool_definitions();
        assert_eq!(tools.len(), 6);

        let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"list_files"));
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"code_search"));
        assert!(names.contains(&"beads"));
    }
}
