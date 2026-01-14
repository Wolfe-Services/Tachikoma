# Spec 554: TUI Loop Runner

**Phase:** 27 - Ralph Integration  
**Status:** Planned  
**Priority:** P0 - Critical  
**Dependencies:** 553 (Quickstart)
**Inspired By:** Ralph TUI's split-pane terminal interface

## Overview

Upgrade the current `ralph` CLI tool from streaming text output to a proper TUI with split-pane layout, progress indicators, and keyboard controls.

## Problem Statement

Current `ralph` tool outputs:
```
--- Iteration 1 ---
[Executing tool: read_file]
...streaming text...
```

Ralph TUI shows:
```
┌─ Tasks ───────────────────┐┌─ Output ─────────────────────────────────────┐
│ ✓ 001 Project Structure   ││ [12:34:56] Reading spec file...              │
│ → 002 Rust Workspace      ││ [12:34:57] Found 5 acceptance criteria       │
│ ○ 003 Electron Shell      ││ [12:34:58] Implementing criterion 1...       │
│ ○ 004 Svelte Integration  ││                                              │
│                           ││ > Editing crates/tachikoma-core/src/lib.rs   │
├───────────────────────────┤│                                              │
│ Progress: 25% ████░░░░░░  ││                                              │
│ Tokens: 45k/150k          ││                                              │
│ Cost: $0.42               ││                                              │
└───────────────────────────┘└──────────────────────────────────────────────┘
 [p]ause  [d]ashboard  [q]uit  [?]help
```

## Acceptance Criteria

- [ ] Implement TUI using `ratatui` crate
- [ ] Left panel: Task/spec list with status indicators (✓ → ○)
- [ ] Right panel: Live streaming agent output
- [ ] Bottom bar: Keyboard shortcuts
- [ ] Progress bar showing spec/criteria completion
- [ ] Token usage gauge with redline indicator
- [ ] Cost tracking display
- [ ] Keyboard controls: p(pause), d(dashboard), q(quit), ?(help)
- [ ] Support for resizable panes
- [ ] History scrollback in output panel

## Keyboard Controls

| Key | Action |
|-----|--------|
| `p` | Pause/Resume execution |
| `d` | Toggle dashboard view |
| `i` | Toggle iteration history |
| `l` | Show full log |
| `q` | Quit (with confirmation if running) |
| `?` | Show help |
| `↑/↓` | Scroll task list |
| `PgUp/PgDn` | Scroll output |
| `Tab` | Switch focus between panes |

## Layout Modes

### Default: Split View
```
┌──────────┬────────────────────────────────────────────┐
│  Tasks   │              Agent Output                  │
│          │                                            │
│          │                                            │
├──────────┴────────────────────────────────────────────┤
│ Progress | Tokens | Cost | Shortcuts                  │
└───────────────────────────────────────────────────────┘
```

### Dashboard View (toggle with 'd')
```
┌─────────────────────────────────────────────────────────┐
│                    Session Dashboard                     │
├─────────────────────────────────────────────────────────┤
│ Specs: 45/100 (45%)      │ Criteria: 234/500 (47%)     │
├─────────────────────────────────────────────────────────┤
│ Session Stats            │ Cost Breakdown               │
│ ─────────────            │ ──────────────               │
│ Started: 2h 34m ago      │ Input:  $12.34               │
│ Iterations: 156          │ Output: $45.67               │
│ Reboots: 3               │ Total:  $58.01               │
│ Commits: 12              │                              │
├─────────────────────────────────────────────────────────┤
│ Token Usage              │ [████████████░░░░░░░] 67%    │
│ 100k / 150k              │                              │
└─────────────────────────────────────────────────────────┘
```

## Implementation Details

### Dependencies

Add to `tools/ralph/Cargo.toml`:
```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.28"
```

### File Structure

```
tools/ralph/src/
├── main.rs              # Entry point, CLI parsing
├── tui/
│   ├── mod.rs           # TUI module
│   ├── app.rs           # App state machine
│   ├── ui.rs            # UI rendering
│   ├── widgets/
│   │   ├── task_list.rs
│   │   ├── output_panel.rs
│   │   ├── progress_bar.rs
│   │   ├── token_gauge.rs
│   │   └── status_bar.rs
│   └── events.rs        # Keyboard event handling
├── claude_client.rs     # (existing)
├── spec_parser.rs       # (existing)
└── primitives.rs        # (existing)
```

### App State

```rust
pub struct App {
    // View state
    current_view: View,
    selected_task: usize,
    output_scroll: usize,
    
    // Execution state
    is_running: bool,
    is_paused: bool,
    current_spec: Option<ParsedSpec>,
    
    // Metrics
    iterations: usize,
    total_tokens: u32,
    total_cost: f64,
    
    // Output buffer
    output_lines: Vec<OutputLine>,
}

enum View {
    Split,
    Dashboard,
    Help,
}
```

### Output Streaming

Modify `claude_client.rs` to send structured events:

```rust
pub enum LoopEvent {
    IterationStart(usize),
    ToolCall { name: String, input: Value },
    ToolResult { name: String, output: String, success: bool },
    Text(String),
    TokenUpdate { input: u32, output: u32 },
    SpecComplete(u32),
    Redline,
}
```

## Testing

- Test TUI renders correctly in various terminal sizes
- Test keyboard controls work
- Test pause/resume doesn't lose state
- Test graceful exit on Ctrl+C
- Verify output scrollback works

## References

- Ralph TUI's interface design
- `ratatui` examples: https://ratatui.rs/examples/
- Current ralph tool: `tools/ralph/src/main.rs`
