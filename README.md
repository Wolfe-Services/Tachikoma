# Ralph Wiggum Loop Harness

> "It's not that hard to build a coding agent. It's 300 lines of code running in a loop with LLM tokens. The model does all the heavy lifting."
> — Geoffrey Huntley

The Ralph Wiggum Loop is an agentic coding harness that autonomously implements tasks from the **beads issue tracker**, one at a time, with fresh contexts.

## Quick Start

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the harness
cargo build --release

# Set your API key
export ANTHROPIC_API_KEY=your-key-here

# Point at a beads-tracked project
./target/release/ralph --project /path/to/project status
./target/release/ralph --project /path/to/project next
./target/release/ralph --project /path/to/project run
./target/release/ralph --project /path/to/project loop
```

## Commands

| Command | Description |
|---------|-------------|
| `status` | Show current progress (completed/total tasks, ready count) |
| `next` | Show the next unblocked task to implement |
| `list [--all]` | List ready tasks (or all open tasks with --all) |
| `show <issue-id>` | Show details of a specific task |
| `run [--issue ID]` | Run once (implement one task) |
| `loop` | Run continuously until all tasks complete |
| `tui` | Run with split-pane terminal interface |

## How It Works

1. **Query Beads**: Runs `bd ready` to find unblocked tasks
2. **Find Next Task**: Locates the first ready task with open status
3. **Fresh Context**: Each task gets a clean Claude context (no pollution)
4. **Six Primitives**: Uses `read_file`, `list_files`, `bash`, `edit_file`, `code_search`, `beads`
5. **Progress Tracking**: Claude uses `beads` tool to update task status
6. **Auto-Sync**: Syncs beads state and commits changes after each implementation

## The Six Primitives

From Geoffrey's experience: **more tools = worse outcomes**. We implement exactly six:

| Primitive | Purpose |
|-----------|---------|
| `read_file` | Read file contents |
| `list_files` | List directory contents |
| `bash` | Execute shell commands (with timeout) |
| `edit_file` | Modify files (unique match required) |
| `code_search` | Ripgrep wrapper for pattern search |
| `beads` | Issue tracker operations (show, update, close, sync) |

### Beads Tool Actions

Claude can interact with the issue tracker:

```
beads action="ready"                           # List unblocked tasks
beads action="show" task_id="project-abc"      # View task details
beads action="update" task_id="..." status="in_progress"  # Update status
beads action="close" task_id="..." reason="Implemented X" # Close task
beads action="sync"                            # Sync beads changes to git
```

## Configuration

### Environment Variables

```bash
export ANTHROPIC_API_KEY=sk-ant-...   # Required
```

### CLI Options

```bash
ralph --project /path/to/project run \
  --issue fulcrum-ra4 \          # Specific task ID (optional)
  --max-iterations 50 \          # Per-task iteration limit
  --redline 150000 \             # Token limit before fresh context
  --no-sync                      # Skip auto-sync

ralph --project /path/to/project loop \
  --max-iterations 50 \          # Per-task iteration limit
  --max-tasks 10 \               # Total tasks to process
  --fail-streak 3 \              # Stop after N consecutive failures
  --no-sync                      # Skip auto-sync
```

## Key Principles

### One Context = One Task
> "My #1 recommendation is to use a context window for one task, and one task only."

Each task implementation gets a fresh context. If stuck, reboot.

### Don't Redline the Context
> "Red is bad because it results in audio clipping and muddy mixes. The same applies to LLMs."

The harness monitors token usage and forces a fresh context when approaching the limit (~150k tokens).

### Beads as State
The beads issue tracker is the source of truth:
- Task status tracks progress
- Descriptions contain acceptance criteria
- Dependencies determine work order (via `bd ready`)

## Project Structure

```
ralph_loop/
├── Cargo.toml            # Rust dependencies
├── README.md             # This file
└── src/
    ├── main.rs           # CLI entry point and loop runner
    ├── task_parser.rs    # Beads integration
    ├── primitives.rs     # Six core tools
    ├── claude_client.rs  # Claude API with streaming
    ├── git.rs            # Auto-commit functionality
    └── tui/              # Terminal UI components
```

## Integration with Beads

Your project needs a `.beads/` directory (run `bd init` to create one).

Tasks should have acceptance criteria in their description:

```markdown
## Description

Implement the user login flow.

## Acceptance Criteria

- [ ] Create login form component
- [ ] Add password validation
- [ ] Integrate with auth service
```

Ralph will:
1. Read the task description
2. Implement the requirements
3. Use `beads action="close"` when complete
4. Auto-commit with message `task(project-xyz): Task Title`

## Troubleshooting

### "No ready tasks found"
All tasks have unresolved dependencies. Check `bd ready` output.

### "Context redlining"
The context window is nearly full. Ralph will reboot with fresh context.

### "old_string not unique"
The `edit_file` tool requires unique matches. Add more context to the search string.

### Consecutive failures
If Ralph fails multiple times in a row, it stops to prevent spiraling. Check:
1. Are tests passing?
2. Is the task description clear?
3. Are dependencies met?

## Economics

Running Ralph continuously costs approximately:
- **Sonnet 4**: ~$10/hour (fast, agentic)
- **Opus 4.5**: ~$50/hour (deep reasoning)

For task implementation, Sonnet is recommended (action-biased).

## Credits

Based on Geoffrey Huntley's "Ralph Wiggum Loop from First Principles" talk.

> "Tachikoma on the case!"
