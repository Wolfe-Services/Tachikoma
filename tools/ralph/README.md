# Ralph Wiggum Loop Harness

> "It's not that hard to build a coding agent. It's 300 lines of code running in a loop with LLM tokens. The model does all the heavy lifting."
> — Geoffrey Huntley

The Ralph Wiggum Loop is an agentic coding harness that autonomously implements specs from THE PIN (specs/README.md), one at a time, with fresh contexts.

## Quick Start

### Using the Shell Script (Requires Claude CLI)

```bash
# Install Claude CLI first
npm install -g @anthropic-ai/claude-code

# Run the harness
cd /path/to/tachikoma
./tools/ralph/ralph.sh status   # Show progress
./tools/ralph/ralph.sh next     # Show next spec
./tools/ralph/ralph.sh run      # Implement one spec
./tools/ralph/ralph.sh loop     # Run continuously
```

### Using the Rust Implementation (Recommended)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the harness
cd tools/ralph
cargo build --release

# Set your API key
export ANTHROPIC_API_KEY=your-key-here

# Run
./target/release/ralph status
./target/release/ralph next
./target/release/ralph run
./target/release/ralph loop
```

## Commands

| Command | Description |
|---------|-------------|
| `status` | Show current progress (completed/total specs) |
| `next` | Show the next spec to implement |
| `run [--spec ID]` | Run once (implement one spec) |
| `loop` | Run continuously until all specs complete |
| `list [--incomplete]` | List all specs |
| `validate` | Validate specs structure |

## How It Works

1. **Navigate THE PIN**: Parses `specs/README.md` to find specs in order
2. **Find Next Task**: Locates the first spec with unchecked acceptance criteria
3. **Fresh Context**: Each spec gets a clean Claude context (no pollution)
4. **Five Primitives**: Uses only `read_file`, `list_files`, `bash`, `edit_file`, `code_search`
5. **Progress Tracking**: Updates checkboxes as criteria are met
6. **Auto-Commit**: Commits changes after each successful implementation

## The Five Primitives

From Geoffrey's experience: **more tools = worse outcomes**. We implement exactly five:

| Primitive | Purpose |
|-----------|---------|
| `read_file` | Read file contents |
| `list_files` | List directory contents |
| `bash` | Execute shell commands (with timeout) |
| `edit_file` | Modify files (unique match required) |
| `code_search` | Ripgrep wrapper for pattern search |

## Configuration

### Environment Variables

```bash
export ANTHROPIC_API_KEY=sk-ant-...   # Required
export PROJECT_ROOT=/path/to/project   # Optional, defaults to pwd
export MAX_ITERATIONS=100              # Max loop iterations
export STOP_ON_FAIL_STREAK=3           # Stop after N failures
```

### Loop Options

```bash
# Rust implementation
ralph loop \
  --max-iterations 50 \      # Per-spec iteration limit
  --max-specs 10 \           # Total specs to process
  --fail-streak 3 \          # Stop after N consecutive failures
  --no-commit                # Skip auto-commit
```

## Key Principles

### One Context = One Task
> "My #1 recommendation is to use a context window for one task, and one task only."

Each spec implementation gets a fresh context. If stuck, reboot.

### Don't Redline the Context
> "Red is bad because it results in audio clipping and muddy mixes. The same applies to LLMs."

The harness monitors token usage and warns when approaching the limit (~150k tokens).

### File System as State
The spec files are the source of truth:
- Checkboxes track progress
- Implementation plans cite code locations
- README.md is the master index (THE PIN)

## Project Structure

```
tools/ralph/
├── Cargo.toml           # Rust dependencies
├── ralph.sh             # Shell script fallback
├── README.md            # This file
└── src/
    ├── main.rs          # CLI entry point and loop runner
    ├── spec_parser.rs   # THE PIN navigation
    ├── primitives.rs    # Five core tools
    ├── claude_client.rs # Claude API with streaming
    └── git.rs           # Auto-commit functionality
```

## Integration with Specs

Specs should have this structure for Ralph to work:

```markdown
# 001 - Spec Name

## Acceptance Criteria

- [ ] First criterion
- [ ] Second criterion
- [ ] Third criterion

## Implementation Details
...
```

Ralph will:
1. Read the spec
2. Implement each unchecked criterion
3. Update `- [ ]` to `- [x]` when done
4. Auto-commit with message `spec(001): implement Spec Name`

## Troubleshooting

### "Context redlining"
The context window is nearly full. Ralph will warn and may need a fresh start.

### "old_string not unique"
The `edit_file` tool requires unique matches. Add more context to the search string.

### Consecutive failures
If Ralph fails multiple times in a row, it stops to prevent spiraling. Check:
1. Are tests passing?
2. Is the spec clear?
3. Are dependencies met?

## Economics

Running Ralph continuously costs approximately:
- **Sonnet 4**: ~$10/hour (fast, agentic)
- **Opus 4.5**: ~$50/hour (deep reasoning)

For spec implementation, Sonnet is recommended (action-biased).

## Credits

Based on Geoffrey Huntley's "Ralph Wiggum Loop from First Principles" talk and the Tachikoma design philosophy.

> "Tachikoma on the case!"
