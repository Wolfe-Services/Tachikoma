# Ralph Loop Improvement Analysis

**Based on analysis of:**
1. Loom CLI (`ghuntley/loom/crates/loom-cli/`)
2. Loom CLI Tools (`ghuntley/loom/crates/loom-cli-tools/`)  
3. Ralph TUI (`subsy/ralph-tui/`)

---

## Executive Summary

Your Ralph Loop has solid foundations but burns context on exploration because:
1. **Primitives lack hard limits** - Files can be 100KB, `list_files` recursive can return 200+ entries
2. **System prompt is passive** - Tells agent to "code first" but doesn't enforce it
3. **No action tracking** - Can't detect exploration spirals (3+ reads with no edits)
4. **No codebase map seeding** - Agent always starts from scratch

**Key insight from ralph-tui**: They inject **previous progress** and **codebase patterns** into each iteration, giving the agent context without burning tokens on exploration.

---

## 1. Loop Architecture Insights (from Loom CLI)

### 1.1 What Loom Does Differently

Loom's main loop has several key patterns we should adopt:

```rust
// From loom-cli/src/main.rs - Tool outcome tracking
let mut tool_outcomes: Vec<(String, bool)> = Vec::new();
for tool_call in &tool_calls {
    let outcome = execute_tool(tool_registry, tool_call, tool_ctx).await;
    let succeeded = matches!(&outcome, ToolExecutionOutcome::Success { .. });
    tool_outcomes.push((tool_call.tool_name.clone(), succeeded));
    // ... add to messages ...
}

// After tool calls complete - auto-commit if edit_file or bash ran
if !tool_calls.is_empty() {
    if let Some(auto_commit_svc) = auto_commit_service {
        let completed: Vec<CompletedToolInfo> = tool_outcomes
            .iter()
            .map(|(name, succeeded)| CompletedToolInfo {
                tool_name: name.clone(),
                succeeded: *succeeded,
            })
            .collect();
        run_auto_commit(auto_commit_svc, workspace, &completed).await;
    }
}
```

**Pattern 1: Auto-commit after edit_file/bash**
- They track which tools ran and auto-commit when `edit_file` or `bash` succeeds
- Your Ralph commits at task end; consider commit-per-edit for recovery

**Pattern 2: Git state snapshot every iteration**
```rust
snapshot_git_state(thread, workspace);
thread.touch();
if let Err(e) = thread_store.save(thread).await {
    warn!(error = %e, "failed to save thread");
}
```

### 1.2 Recommended Loop Changes for Ralph

```rust
// In claude_client.rs - Add action tracking
pub struct IterationMetrics {
    pub read_count: u32,
    pub edit_count: u32,
    pub search_count: u32,
    pub bash_count: u32,
}

// Track per-iteration to detect exploration spirals
fn is_exploration_spiral(metrics: &IterationMetrics) -> bool {
    metrics.read_count >= 3 && metrics.edit_count == 0 && metrics.search_count >= 2
}
```

**Inject "nudge" prompt when spiral detected:**
```rust
if is_exploration_spiral(&iteration_metrics) {
    // Add a user message nudging toward action
    messages.push(Message {
        role: Role::User,
        content: vec![ContentBlock::Text {
            text: "[SYSTEM: You've spent 3+ iterations exploring. The task description contains file paths. START CODING NOW with edit_file.]".to_string(),
        }],
    });
}
```

---

## 2. Primitive Improvements (from Loom CLI Tools)

### 2.1 read_file Improvements

**Current Ralph:**
```rust
let max_size = 100_000; // ~100KB
if content.len() > max_size {
    let truncated = &content[..max_size];
    // ... return truncated
}
```

**Loom's approach:**
```rust
// From loom-cli-tools/src/read_file.rs
const DEFAULT_MAX_BYTES: u64 = 1024 * 1024; // 1MB

// They also return a "truncated" boolean in the result
let result = ReadFileResult {
    path: canonical_path,
    contents,
    truncated, // <-- Agent knows if it got partial content
};
```

**Recommended change - Add line limiting:**
```rust
#[derive(Debug, Deserialize)]
struct ReadFileArgs {
    path: String,
    max_bytes: Option<u64>,     // NEW: soft limit
    start_line: Option<usize>,  // NEW: for targeted reads
    end_line: Option<usize>,    // NEW: for targeted reads
}

// Tool definition
{
    "name": "read_file",
    "description": "Read file contents. For large files, use start_line/end_line to read specific sections instead of the whole file.",
    "input_schema": {
        "type": "object",
        "properties": {
            "path": { "type": "string" },
            "start_line": { 
                "type": "integer",
                "description": "Read from this line (1-indexed). Use with end_line for targeted reads."
            },
            "end_line": { 
                "type": "integer",
                "description": "Read until this line (inclusive)."
            }
        },
        "required": ["path"]
    }
}
```

### 2.2 list_files - Critical for Context Management

**Current Ralph (already decent):**
```rust
const IGNORED_DIRS: &[&str] = &[
    "node_modules", ".git", "target", ".next", "dist", "__pycache__",
    // ...
];

// Flat: limit 100 entries
// Recursive: limit 200 entries
```

**Loom's stricter approach:**
```rust
// From loom-cli-tools/src/list_files.rs
const DEFAULT_MAX_RESULTS: usize = 1000;

// FLAT ONLY - no recursive option in their schema!
// This forces agents to navigate incrementally
```

**Recommended change - Remove or heavily limit recursive:**
```rust
async fn list_files(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let path_str = /* ... */;
    let recursive = input.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
    
    // AGGRESSIVE LIMITING for recursive
    if recursive {
        // Only allow recursive in specific directories
        let depth = input.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(2);
        let max_entries = 50; // Much stricter than 200
        
        // Emit warning if truncated
        if entries.len() >= max_entries {
            entries.push("[TRUNCATED - Use targeted paths instead of recursive listing]".to_string());
        }
    }
    // ...
}
```

### 2.3 code_search - Already Good, Add Summary Mode

**Add a summary option that just returns file paths + counts:**
```rust
#[derive(Debug, Deserialize)]
struct CodeSearchArgs {
    pattern: String,
    path: Option<String>,
    file_pattern: Option<String>,
    max_results: Option<usize>,
    summary_only: Option<bool>, // NEW: just return files + match counts
}

// When summary_only = true, output like:
// "Found 15 matches in 4 files:
//   src/main.rs: 8 matches
//   src/lib.rs: 4 matches
//   src/utils.rs: 2 matches
//   src/test.rs: 1 match"
```

### 2.4 bash - Add Command Allowlist/Blocklist

**Current risk**: Agent can run any bash command, potentially wasting context on exploratory commands like `find` or `grep -r`.

```rust
const ALLOWED_PATTERNS: &[&str] = &[
    "cargo build", "cargo test", "cargo check", "cargo fmt",
    "npm run", "npm test", "npm build",
    "dotnet build", "dotnet test",
    "git status", "git diff", "git add", "git commit",
    "bd ", // beads commands
];

const BLOCKED_PATTERNS: &[&str] = &[
    "find ", "locate ", // Use list_files instead
    "grep -r", "rg ",   // Use code_search instead
    "cat ",             // Use read_file instead
];

async fn bash(input: &serde_json::Value, project_root: &Path) -> ToolResult {
    let command = /* ... */;
    
    // Warn on blocked patterns
    for pattern in BLOCKED_PATTERNS {
        if command.starts_with(pattern) || command.contains(&format!(" {}", pattern)) {
            return ToolResult::error(format!(
                "Blocked command '{}'. Use the dedicated tool instead:\n\
                 - find/locate → list_files\n\
                 - grep/rg → code_search\n\
                 - cat → read_file",
                pattern
            ));
        }
    }
    // ...
}
```

---

## 3. Beads Integration (from ralph-tui)

### 3.1 Progress Injection Pattern

**This is the most impactful pattern from ralph-tui.**

They maintain a `progress.md` file and inject "recent progress" into each prompt:

```typescript
// From ralph-tui/src/engine/index.ts
async function buildPrompt(
  task: TrackerTask,
  config: RalphConfig,
  tracker?: TrackerPlugin
): Promise<string> {
  // Load recent progress for context (last 5 iterations)
  const recentProgress = await getRecentProgressSummary(config.cwd, 5);

  // Load codebase patterns from progress.md (if any exist)
  const codebasePatterns = await getCodebasePatternsForPrompt(config.cwd);
  
  // Inject into template
  const extendedContext = {
    recentProgress,
    codebasePatterns,
    prd: prdContext ?? undefined,
  };
  // ...
}
```

**Their template includes:**
```handlebars
{{#if recentProgress}}
## Previous Progress
{{recentProgress}}
{{/if}}

{{#if codebasePatterns}}
## Codebase Patterns (Study These First)
{{codebasePatterns}}
{{/if}}
```

**Recommended implementation for Ralph:**

```rust
// New file: src/progress.rs

use std::path::Path;
use anyhow::Result;

const PROGRESS_FILE: &str = ".ralph/progress.md";

/// Load recent progress summary (last N entries)
pub fn load_recent_progress(project_root: &Path, max_entries: usize) -> Result<String> {
    let progress_path = project_root.join(PROGRESS_FILE);
    
    if !progress_path.exists() {
        return Ok(String::new());
    }
    
    let content = std::fs::read_to_string(&progress_path)?;
    
    // Parse markdown entries (## headers separate entries)
    let entries: Vec<&str> = content.split("\n## ").collect();
    
    // Take last N entries
    let recent: Vec<&str> = entries.iter()
        .rev()
        .take(max_entries)
        .rev()
        .cloned()
        .collect();
    
    if recent.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("## {}", recent.join("\n## ")))
    }
}

/// Append a progress entry after completing a task
pub fn append_progress(
    project_root: &Path, 
    task_id: &str,
    summary: &str,
    files_changed: &[String],
) -> Result<()> {
    let progress_path = project_root.join(PROGRESS_FILE);
    
    // Ensure directory exists
    if let Some(parent) = progress_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let entry = format!(
        "\n\n## {} - {}\n{}\n\n**Files:** {}\n---",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        task_id,
        summary,
        files_changed.join(", ")
    );
    
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&progress_path)?;
    
    use std::io::Write;
    writeln!(file, "{}", entry)?;
    
    Ok(())
}
```

**Update system prompt to use it:**
```rust
fn build_system_prompt(project_root: &PathBuf) -> String {
    let recent_progress = progress::load_recent_progress(project_root, 3)
        .unwrap_or_default();
    
    format!(
        r#"You are an AI coding assistant implementing tasks from a beads issue tracker.

## CRITICAL: Code First, Explore Minimally

{recent_progress_section}

## Previous Work (Study This First)
{recent_progress}

## Project Root
{project_root}
...
"#,
        recent_progress_section = if recent_progress.is_empty() {
            ""
        } else {
            "Previous iterations have documented patterns and learnings below. Use them!"
        },
        recent_progress = if recent_progress.is_empty() {
            "No previous progress recorded.".to_string()
        } else {
            recent_progress
        },
        project_root = project_root.display()
    )
}
```

### 3.2 Codebase Map Seeding

**If you have CODEMAP.md**, seed it into the system prompt:

```rust
fn load_codebase_summary(project_root: &Path) -> String {
    // Try multiple possible locations
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
                let max_chars = 8000;
                if content.len() > max_chars {
                    return format!(
                        "{}...\n[CODEMAP TRUNCATED - Full map at {}]",
                        &content[..max_chars],
                        candidate
                    );
                }
                return content;
            }
        }
    }
    
    String::new()
}
```

### 3.3 Task Completion Signal

**ralph-tui uses a completion signal pattern:**
```typescript
const PROMISE_COMPLETE_PATTERN = /<promise>\s*COMPLETE\s*<\/promise>/i;
```

This is explicit and unambiguous. Your current approach relies on `beads close`, which is good but the agent might forget. Consider adding this as an alternative:

```rust
// In claude_client.rs - Check for completion signal in text
fn check_completion_signal(text: &str) -> bool {
    let pattern = regex::Regex::new(r"(?i)<promise>\s*COMPLETE\s*</promise>").unwrap();
    pattern.is_match(text)
}

// In the loop
for block in &response.content {
    if let ContentBlock::Text { text } = block {
        if check_completion_signal(text) {
            // Agent is declaring completion - verify and close
            return Ok(LoopResult {
                stop_reason: StopReason::Completed,
                // ...
            });
        }
    }
}
```

---

## 4. Prompt Engineering

### 4.1 Action-Oriented System Prompt

**Problem**: Your current system prompt says "code first" but doesn't enforce it.

**ralph-tui's approach** - Explicit workflow with numbered steps:

```
## Workflow
1. Study the PRD context above to understand the bigger picture
2. Study `.ralph-tui/progress.md` to understand patterns and learnings
3. Implement the requirements (stay on current branch)
4. Run your project's quality checks (typecheck, lint, etc.)
5. Commit: `feat: {{taskId}} - {{taskTitle}}`
6. Close the bead: `bd close {{taskId}} --reason "Brief description"`
7. Document learnings (see below)
8. Signal completion
```

**Recommended rewrite for Ralph:**

```rust
fn build_system_prompt(project_root: &PathBuf) -> String {
    let codemap = load_codebase_summary(project_root);
    let recent_progress = progress::load_recent_progress(project_root, 3)
        .unwrap_or_default();

    format!(r#"You are Ralph, an AI coding assistant. Your mission: implement tasks efficiently with minimal exploration.

## CRITICAL RULES (Violations will trigger intervention)

1. **3-iteration rule**: You MUST produce an edit_file call within 3 iterations or explain why
2. **No recursive exploration**: Use targeted file reads, not recursive list_files
3. **Task description is truth**: File paths in the task description are accurate - trust them
4. **One search, then code**: Maximum ONE code_search per pattern, then implement

## Codebase Overview
{codemap_section}

## Recent Progress (From Previous Tasks)
{progress_section}

## Project Root
{project_root}

## Available Tools
| Tool | When to Use |
|------|-------------|
| read_file | Read specific file (use start_line/end_line for large files) |
| list_files | List ONE directory (never recursive for exploration) |
| code_search | Find pattern (ONE search per concept) |
| edit_file | Create/modify files (empty old_string = new file) |
| bash | Build/test/git commands (NOT for file exploration) |
| beads | Task management (close when done) |

## Anti-Patterns (DO NOT)
- ❌ Reading more than 3 files before making an edit
- ❌ Using list_files recursive for "exploring the codebase"
- ❌ Running bash commands like `find`, `grep -r`, `cat` (use dedicated tools)
- ❌ Reading the entire project structure before starting

## Completion
When task is done: `beads action="close" task_id="<id>" reason="<summary>"`
"#,
        codemap_section = if codemap.is_empty() {
            "No codemap available. Read task description for file locations.".to_string()
        } else {
            codemap
        },
        progress_section = if recent_progress.is_empty() {
            "No previous progress. You're starting fresh.".to_string()
        } else {
            recent_progress
        },
        project_root = project_root.display()
    )
}
```

### 4.2 Task Prompt with Explicit Action Plan

```rust
fn build_task_prompt(parsed: &ParsedTask) -> String {
    // Count incomplete criteria
    let incomplete: Vec<_> = parsed.acceptance_criteria
        .iter()
        .filter(|ac| !ac.completed)
        .collect();
    
    let criteria_section = if incomplete.is_empty() {
        "All criteria complete. Verify and close the task.".to_string()
    } else {
        incomplete.iter()
            .map(|ac| format!("- [ ] {}", ac.text))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(r#"## TASK: {id} - {title}

### Description
{description}

### Remaining Criteria
{criteria}

### YOUR EXECUTION PLAN (Follow exactly)

**Iteration 1**: 
- ONE code_search to find similar pattern (if needed)
- OR read ONE file mentioned in description

**Iteration 2-3**:
- Create/edit files with edit_file
- empty old_string = create new file

**Final**:
- Run: `bash command="cargo build"` (or appropriate build command)
- Close: `beads action="close" task_id="{id}" reason="<what was done>"`

### IMPORTANT
The description tells you exactly which files to create/edit. TRUST IT.
Do not explore - START CODING.
"#,
        id = parsed.task.id,
        title = parsed.task.title,
        description = parsed.task.description,
        criteria = criteria_section,
    )
}
```

---

## 5. Concrete Code Snippets to Add

### 5.1 Exploration Detection and Intervention

Add to `claude_client.rs`:

```rust
/// Metrics for detecting exploration spirals
#[derive(Debug, Default)]
struct IterationMetrics {
    read_file_count: u32,
    list_files_count: u32,
    code_search_count: u32,
    edit_file_count: u32,
    bash_count: u32,
    beads_count: u32,
}

impl IterationMetrics {
    fn record_tool(&mut self, name: &str) {
        match name {
            "read_file" => self.read_file_count += 1,
            "list_files" => self.list_files_count += 1,
            "code_search" => self.code_search_count += 1,
            "edit_file" => self.edit_file_count += 1,
            "bash" => self.bash_count += 1,
            "beads" => self.beads_count += 1,
            _ => {}
        }
    }
    
    fn is_exploration_heavy(&self) -> bool {
        let exploration = self.read_file_count + self.list_files_count + self.code_search_count;
        let action = self.edit_file_count + self.bash_count;
        exploration >= 5 && action == 0
    }
    
    fn total_exploration(&self) -> u32 {
        self.read_file_count + self.list_files_count + self.code_search_count
    }
}

// In run_agentic_loop, add metrics tracking:
let mut session_metrics = IterationMetrics::default();

// After executing tools:
for (id, name, input) in &tool_calls {
    session_metrics.record_tool(&name);
    // ... execute tool ...
}

// Check for exploration spiral every 3 iterations
if iterations % 3 == 0 && session_metrics.is_exploration_heavy() {
    // Inject intervention message
    messages.push(Message {
        role: Role::User,
        content: vec![ContentBlock::Text {
            text: format!(
                "[INTERVENTION] You've made {} exploration calls but 0 edits.\n\
                 The task description contains file paths. Create files NOW with:\n\
                 edit_file path=\"<path>\" old_string=\"\" new_string=\"<content>\"\n\
                 NO MORE EXPLORATION.",
                session_metrics.total_exploration()
            ),
        }],
    });
}
```

### 5.2 Parallel Tool Calls

Loom doesn't show parallel execution, but your current implementation processes sequentially. Consider:

```rust
// Current (sequential):
for (id, name, input) in tool_calls {
    let result = execute_tool(&name, &input, &self.project_root).await;
    tool_results.push(/* ... */);
}

// Parallel (for independent calls like multiple reads):
let tool_futures: Vec<_> = tool_calls.iter()
    .map(|(id, name, input)| {
        let name = name.clone();
        let input = input.clone();
        let project_root = self.project_root.clone();
        async move {
            (id.clone(), execute_tool(&name, &input, &project_root).await)
        }
    })
    .collect();

let results = futures::future::join_all(tool_futures).await;
```

**Note**: Only do this for read operations. Edits should remain sequential.

### 5.3 Rate Limit Detection (from ralph-tui)

Add rate limit detection to avoid burning tokens on retries:

```rust
// In primitives.rs or a new rate_limit.rs

pub struct RateLimitDetector {
    patterns: Vec<regex::Regex>,
}

impl RateLimitDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                regex::Regex::new(r"(?i)rate[- ]?limit").unwrap(),
                regex::Regex::new(r"(?i)429").unwrap(),
                regex::Regex::new(r"(?i)too many requests").unwrap(),
                regex::Regex::new(r"(?i)quota[- ]?exceeded").unwrap(),
                regex::Regex::new(r"(?i)\boverloaded\b").unwrap(),
            ],
        }
    }
    
    pub fn is_rate_limited(&self, output: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(output))
    }
    
    pub fn extract_retry_after(&self, output: &str) -> Option<u64> {
        let re = regex::Regex::new(r"retry[- ]?after[:\s]+(\d+)").unwrap();
        re.captures(output)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}

// Use in API call handling:
if rate_detector.is_rate_limited(&error_message) {
    if let Some(wait_secs) = rate_detector.extract_retry_after(&error_message) {
        tracing::warn!("Rate limited. Waiting {} seconds...", wait_secs);
        tokio::time::sleep(Duration::from_secs(wait_secs)).await;
    }
}
```

---

## 6. Summary: Priority Improvements

### High Priority (Do First)

1. **Add progress injection** - Load `.ralph/progress.md` into system prompt
2. **Add codebase map seeding** - Load `CODEMAP.md` into system prompt  
3. **Add exploration detection** - Track read/edit ratio, intervene at 5:0
4. **Limit recursive list_files** - Drop to 50 entries max, add warning

### Medium Priority

5. **Add line-range support to read_file** - `start_line`/`end_line` params
6. **Block exploratory bash** - Reject `find`, `grep -r`, `cat` in favor of tools
7. **Add summary mode to code_search** - Just file paths + counts
8. **Rewrite system prompt** - Explicit rules table, anti-patterns section

### Low Priority (Nice to Have)

9. **Parallel tool execution** - For multiple read operations
10. **Rate limit detection** - Parse error messages for retry-after
11. **Auto-commit per edit** - Like Loom's pattern
12. **Completion signal** - `<promise>COMPLETE</promise>` alternative

---

## 7. Files to Modify

```
ralph_loop/src/
├── main.rs           # Update build_system_prompt, build_task_prompt
├── primitives.rs     # Add limits to read_file, list_files; block exploratory bash
├── claude_client.rs  # Add IterationMetrics, exploration detection
├── progress.rs       # NEW: Progress loading/saving
└── rate_limit.rs     # NEW: Rate limit detection
```

---

## References

- Loom CLI: https://github.com/ghuntley/loom/tree/trunk/crates/loom-cli
- Loom CLI Tools: https://github.com/ghuntley/loom/tree/trunk/crates/loom-cli-tools
- Ralph TUI: https://github.com/subsy/ralph-tui
