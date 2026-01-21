# Handover: Continue Ralph Loop Research & Refinement

**Date**: 2026-01-21  
**Last Commit**: `27da147` - feat: Wire progress tracking into task completion  
**Status**: Progress tracking wired in, thresholds evaluated, ready for real-world testing

---

## What Was Done

### Implemented Improvements (Commits 6850baa, 27da147)

1. **Progress Injection & Recording** (`src/progress.rs`)
   - Loads `.ralph/progress.md` for inter-session context
   - Loads `CODEMAP.md` / `CODEMAP_COMPACT.md` for codebase overview
   - **NEW**: `append_progress()` now wired into task completion
   - **NEW**: `extract_modified_files()` fixed to parse edit_file outputs correctly
   - Progress recorded on: Completed, Redline, and MaxIterations outcomes

2. **Exploration Detection** (`src/claude_client.rs`)
   - `IterationMetrics` tracks tool usage (read_file, list_files, code_search vs edit_file, bash)
   - Detects exploration spirals: 5+ exploration calls with 0 edits
   - Injects intervention message every 3 iterations when spiraling
   - **NEW**: `tool_outputs` tracked in `LoopResult` for progress recording

3. **Stricter Primitives** (`src/primitives.rs`)
   - `list_files` recursive: 200 → 50 entry limit with warning
   - `read_file`: Added `start_line`/`end_line` for targeted reads
   - `bash`: Blocks `find`, `grep -r`, `cat`, `tree`, etc. with helpful errors
   - **EVALUATED**: Bash blocking is appropriate - forces structured tool use

4. **Rewritten Prompts** (`src/main.rs`)
   - System prompt includes codemap, progress, anti-patterns section
   - Task prompt has numbered execution plan (Orient → Implement → Verify)
   - Explicit "3-iteration rule" enforcement
   - **NEW**: Progress recording on task completion/partial completion

---

## Reference Codebases for Continued Research

### 1. Loom CLI (Primary Reference)
- **Location**: External - `ghuntley/loom/crates/loom-cli/`
- **Key Patterns**:
  - Tool outcome tracking with auto-commit
  - Git state snapshots per iteration
  - Parallel tool execution for reads
- **Still to Adopt**:
  - Auto-commit per edit (vs commit at task end)
  - Rate limit detection with retry-after parsing

### 2. Loom CLI Tools
- **Location**: External - `ghuntley/loom/crates/loom-cli-tools/`
- **Key Patterns**:
  - Truncation with explicit boolean in result
  - Flat-only list_files (no recursive in schema)
- **Still to Adopt**:
  - Consider removing recursive option entirely
  - Return `truncated: bool` in tool results

### 3. Ralph TUI
- **Location**: External - `subsy/ralph-tui/`
- **Key Patterns**:
  - Progress.md injection per iteration
  - Codebase patterns extraction
  - `<promise>COMPLETE</promise>` completion signal
- **Still to Adopt**:
  - Completion signal as alternative to beads close
  - PRD context injection

---

## Next Steps for Refinement

### High Priority - Test & Validate

1. **Run Ralph on a real task** and observe:
   - Does the codebase map help?
   - Does exploration detection trigger appropriately?
   - Is progress being recorded correctly?
   - Does progress injection in subsequent runs provide useful context?

2. **Measure token efficiency**:
   - Before: How many tokens to complete a typical task?
   - After: Same task with improvements?
   - Check `.ralph/progress.md` after a few tasks

### Medium Priority - Additional Improvements

3. **Rate limit detection** (from analysis doc):
   ```rust
   pub struct RateLimitDetector {
       patterns: Vec<regex::Regex>,
   }
   
   impl RateLimitDetector {
       pub fn is_rate_limited(&self, output: &str) -> bool { ... }
       pub fn extract_retry_after(&self, output: &str) -> Option<u64> { ... }
   }
   ```

4. **Parallel tool execution for reads**:
   ```rust
   let tool_futures = tool_calls.iter().filter(|t| t.is_read_only()).map(...);
   let results = futures::future::join_all(tool_futures).await;
   ```

### Low Priority - Nice to Have

5. **Auto-commit per edit** (like Loom):
   - Commit immediately after successful `edit_file`
   - Enables recovery on redline

6. **Completion signal alternative**:
   - Detect `<promise>COMPLETE</promise>` in agent output
   - Use as alternative/backup to `beads close`

### ✅ Completed

- ~~Wire `append_progress` into task completion~~ (Done in 27da147)
- ~~Evaluate exploration threshold (5+/0)~~ (Kept as-is, appropriate threshold)
- ~~Evaluate bash blocking~~ (Kept as-is, forces structured tool use)

---

## Files Modified in This Session

| File | Changes |
|------|---------|
| `src/progress.rs` | Progress loading/saving, wired into task completion |
| `src/claude_client.rs` | IterationMetrics, exploration detection, tool_outputs tracking |
| `src/primitives.rs` | Stricter limits on list_files, read_file, bash |
| `src/main.rs` | Progress recording on task completion (all outcomes) |
| `src/main.rs` | Rewrote build_system_prompt, build_task_prompt |
| `docs/IMPROVEMENT_ANALYSIS.md` | Full analysis of Loom CLI, Loom Tools, Ralph TUI |

---

## How to Continue

### Resume Prompt

```
Continue refining the Ralph Loop agentic coding harness.

Location: /Users/mubix/Documents/GitHubCode/ralph_loop/

Current state:
- Commit 6850baa implemented initial context efficiency improvements
- See docs/IMPROVEMENT_ANALYSIS.md for full pattern analysis
- See docs/HANDOVER_CONTINUE_RESEARCH.md for next steps

Tasks:
1. Test the current implementation on a real task
2. Measure token efficiency (before/after)
3. Consider implementing: summary_only for code_search, rate limit detection, wiring append_progress into completion
4. Review if bash blocking is too aggressive

Key files:
- src/progress.rs - Progress injection (load/append)
- src/claude_client.rs - IterationMetrics, exploration detection
- src/primitives.rs - Tool implementations with limits
- src/main.rs - build_system_prompt(), build_task_prompt()
```

---

## Questions Answered

1. **Is 5+ exploration / 0 edits the right threshold for intervention?**
   - ✅ **YES** - Kept at 5+/0. Gives room for legitimate orientation (1-2 reads/searches).
   - Intervention only triggers every 3 iterations AND when spiraling, so not too aggressive.
   - If agent makes even 1 edit, intervention doesn't trigger.

2. **Is the bash blocklist too aggressive?**
   - ✅ **NO** - Kept as-is. Forces structured tool use.
   - `cat` → `read_file` with limits; `grep -r` → `code_search`; `find` → `list_files`
   - Structured tools give bounded output; bash can return unbounded data.

## Open Questions

1. **Should recursive list_files be removed entirely?**
   - Loom doesn't have it
   - Current: 50 entry limit is quite restrictive, may be sufficient

2. **Should we add per-edit auto-commit?**
   - Pro: Recovery on redline
   - Con: Noisy git history
   - Loom does this; Ralph currently commits at task end

---

## Build Commands

```bash
cd /Users/mubix/Documents/GitHubCode/ralph_loop

# Build
RUSTC_WRAPPER="" cargo build --release

# Test
RUSTC_WRAPPER="" cargo test

# Run on a project
./target/release/ralph --project /path/to/project status
./target/release/ralph --project /path/to/project run --issue <task-id>
./target/release/ralph --project /path/to/project loop --max-tasks 5
```

---

**End of Handover**
