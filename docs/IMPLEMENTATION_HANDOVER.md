# Handover Prompt: Implement Ralph Loop Improvements

Copy everything below the line into a new chat.

---

## Context

We have a Rust-based agentic coding loop called **Ralph** that:
- Uses Claude API with 6 primitives: `read_file`, `list_files`, `bash`, `edit_file`, `code_search`, `beads`
- Picks tasks from a Beads issue tracker (`.beads/` directory with YAML files)
- Runs iterations until task is complete or hits redline (context limit)
- **Location**: `/Users/mubix/Documents/GitHubCode/ralph_loop/`

**Problem**: Ralph burns through context on large codebases by over-exploring before coding. It hits redline (~125k tokens) without producing useful code.

**Analysis completed**: We analyzed Loom CLI, Loom CLI Tools, and ralph-tui for patterns. Full analysis is in:
`/Users/mubix/Documents/GitHubCode/ralph_loop/docs/IMPROVEMENT_ANALYSIS.md`

---

## Your Mission: Implement These Improvements

### Priority 1: Progress Injection (New File)

Create `src/progress.rs`:
- Function `load_recent_progress(project_root: &Path, max_entries: usize) -> Result<String>`
  - Loads `.ralph/progress.md`, returns last N entries
- Function `append_progress(project_root: &Path, task_id: &str, summary: &str, files_changed: &[String]) -> Result<()>`
  - Appends entry after task completion

Wire into `main.rs`:
- Call `load_recent_progress()` in `build_system_prompt()`
- Inject result into system prompt under "## Previous Work"

### Priority 2: Exploration Detection (Modify `claude_client.rs`)

Add `IterationMetrics` struct:
```rust
struct IterationMetrics {
    read_file_count: u32,
    list_files_count: u32,
    code_search_count: u32,
    edit_file_count: u32,
    bash_count: u32,
}
```

In `run_agentic_loop`:
- Track tool calls with `metrics.record_tool(&name)`
- Every 3 iterations, check `metrics.is_exploration_heavy()` (5+ exploration, 0 edits)
- If true, inject intervention message into conversation

### Priority 3: Stricter Primitives (Modify `primitives.rs`)

**list_files**:
- Reduce recursive limit from 200 to 50 entries
- Add warning when truncated: `"[TRUNCATED - Use targeted paths]"`

**read_file**:
- Add optional `start_line` and `end_line` parameters to schema
- Implement line-range reading for large files

**bash**:
- Block exploratory commands: `find `, `grep -r`, `cat `
- Return error message pointing to dedicated tools

### Priority 4: Rewrite Prompts (Modify `main.rs`)

**`build_system_prompt()`**:
1. Add codebase map loading (check for `CODEMAP.md` or `CODEMAP_COMPACT.md`)
2. Add recent progress section
3. Add explicit "Anti-Patterns" section listing what NOT to do
4. Add "3-iteration rule" - must produce edit within 3 iterations

**`build_task_prompt()`**:
1. Add numbered execution plan (Iteration 1: search, Iteration 2-3: edit, Final: build+close)
2. Add explicit instruction to trust task description file paths

---

## Key Files to Modify

```
ralph_loop/src/
├── main.rs           # build_system_prompt(), build_task_prompt()
├── primitives.rs     # read_file, list_files, bash limits
├── claude_client.rs  # Add IterationMetrics, exploration detection
└── progress.rs       # NEW FILE
```

---

## Reference: Current Implementation

**System prompt location**: `main.rs` line ~757, function `build_system_prompt()`
**Task prompt location**: `main.rs` line ~818, function `build_task_prompt()`
**Tool definitions**: `primitives.rs` line ~54, function `get_tool_definitions()`
**Tool execution**: `primitives.rs` line ~215, function `execute_tool()`
**Agentic loop**: `claude_client.rs` line ~163, function `run_agentic_loop()`

---

## Success Criteria

1. ✅ Agent receives codebase map in system prompt (if CODEMAP.md exists)
2. ✅ Agent receives recent progress from previous iterations
3. ✅ Exploration spiral (5 reads, 0 edits) triggers intervention message
4. ✅ Recursive list_files limited to 50 entries with warning
5. ✅ Exploratory bash commands blocked with helpful error
6. ✅ System prompt has explicit anti-patterns section

---

## Don't Do

- Don't change the Claude API integration or model selection
- Don't modify beads integration (it works fine)
- Don't add new dependencies unless absolutely necessary
- Don't change the CLI interface

---

Start by reading the full analysis: `/Users/mubix/Documents/GitHubCode/ralph_loop/docs/IMPROVEMENT_ANALYSIS.md`

Then implement in priority order, testing after each change.
