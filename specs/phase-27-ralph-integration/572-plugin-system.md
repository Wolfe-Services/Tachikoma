# Spec 572: Plugin System

**Phase:** 27 - Ralph Integration  
**Status:** Planned  
**Priority:** P1 - High  
**Dependencies:** 551 (Application Shell)
**Inspired By:** Ralph TUI's plugin architecture

## Overview

Create a plugin system for agents, trackers, and templates that allows hot-swapping and user contributions.

## Problem Statement

Current Tachikoma has backend abstraction, but:
- Adding a new agent requires code changes
- Templates are hardcoded in Forge
- No way for users to contribute plugins
- No hot-reload during development

Ralph TUI has:
- `plugins/agents/` - Claude, OpenCode
- `plugins/trackers/` - JSON, Beads
- `prompt-templates/` - Handlebars customization

## Acceptance Criteria

- [x] Define plugin trait interface for agents
- [x] Define plugin trait interface for trackers
- [x] Create `.tachikoma/plugins/` directory structure
- [x] Support plugin discovery and loading at startup
- [x] Add `--agent` CLI flag for runtime agent selection
- [x] Add `--tracker` CLI flag for tracker selection
- [x] Create plugin manifest format (`plugin.yaml`)
- [x] Support Handlebars templates for prompts
- [x] Add plugin validation on load
- [x] Document plugin development guide

## Plugin Types

### 1. Agent Plugins

Agents are LLM backends that execute tool calls.

```rust
// crates/tachikoma-plugin/src/agent.rs
pub trait AgentPlugin: Send + Sync {
    /// Plugin metadata
    fn manifest(&self) -> PluginManifest;
    
    /// Initialize the agent with config
    async fn init(&mut self, config: &AgentConfig) -> Result<()>;
    
    /// Run the agentic loop
    async fn run_loop(
        &self,
        system_prompt: &str,
        task: &str,
        tools: &[ToolDefinition],
        config: &LoopConfig,
    ) -> Result<LoopResult>;
    
    /// Stream events during execution
    fn event_stream(&self) -> impl Stream<Item = LoopEvent>;
}
```

### 2. Tracker Plugins

Trackers manage task/spec state.

```rust
// crates/tachikoma-plugin/src/tracker.rs
pub trait TrackerPlugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    
    /// Get next task to execute
    async fn next_task(&self) -> Result<Option<Task>>;
    
    /// Mark task complete
    async fn complete_task(&self, id: &str) -> Result<()>;
    
    /// Get progress summary
    async fn progress(&self) -> Result<Progress>;
    
    /// Sync with backing store
    async fn sync(&self) -> Result<()>;
}
```

### 3. Template Plugins

Templates customize prompts using Handlebars.

```
.tachikoma/templates/
├── system-prompt.hbs       # Main system prompt
├── task-prompt.hbs         # Per-task prompt
├── critique-prompt.hbs     # Forge critique
└── synthesis-prompt.hbs    # Forge synthesis
```

## Directory Structure

```
.tachikoma/
├── config.yaml
└── plugins/
    ├── agents/
    │   ├── claude/
    │   │   ├── plugin.yaml
    │   │   └── (built-in, no additional files)
    │   ├── opencode/
    │   │   ├── plugin.yaml
    │   │   └── adapter.sh
    │   └── ollama/
    │       ├── plugin.yaml
    │       └── adapter.sh
    ├── trackers/
    │   ├── specs/            # Default spec tracker
    │   │   └── plugin.yaml
    │   ├── beads/
    │   │   └── plugin.yaml
    │   └── json/
    │       └── plugin.yaml
    └── templates/
        ├── default/
        │   ├── system-prompt.hbs
        │   └── task-prompt.hbs
        └── minimal/
            ├── system-prompt.hbs
            └── task-prompt.hbs
```

## Plugin Manifest

```yaml
# plugin.yaml
name: opencode
version: "1.0.0"
type: agent
description: OpenCode AI agent integration

# Requirements
requires:
  - binary: opencode
    version: ">=0.1.0"
  - env: OPENCODE_API_KEY

# Configuration schema
config:
  model:
    type: string
    default: "opencode-large"
  max_tokens:
    type: integer
    default: 8192

# Entry point (for external agents)
adapter: ./adapter.sh
```

## CLI Usage

```bash
# Use specific agent
tachikoma run --agent opencode

# Use specific tracker
tachikoma run --tracker beads

# Use specific template set
tachikoma run --templates minimal

# List available plugins
tachikoma plugins list

# Install plugin from registry (future)
tachikoma plugins install claude-extended

# Validate plugins
tachikoma plugins validate
```

## Handlebars Templates

### System Prompt Template

```handlebars
{{! system-prompt.hbs }}
You are a Tachikoma - a curious, helpful AI coding assistant.

## Project: {{project.name}}
Root: {{project.root}}

## Available Tools
{{#each tools}}
- **{{name}}**: {{description}}
{{/each}}

## Current Mission
{{#if spec}}
Implementing spec {{spec.id}}: {{spec.name}}
Phase: {{spec.phase}}
{{/if}}

{{#if custom_instructions}}
## Custom Instructions
{{custom_instructions}}
{{/if}}
```

### Task Prompt Template

```handlebars
{{! task-prompt.hbs }}
## Mission: Implement Spec {{spec.id}} - {{spec.name}}

### Spec File
{{spec.path}}

### Remaining Criteria
{{#each spec.incomplete_criteria}}
- [ ] {{this}}
{{/each}}

### Instructions
1. Read the spec file first
2. Implement each criterion
3. Mark checkboxes when done
{{#if patterns}}

### Relevant Patterns
{{#each patterns}}
- {{file}}: {{description}}
{{/each}}
{{/if}}
```

## Implementation Details

### Files to Create

1. **`crates/tachikoma-plugin/`** - New plugin crate
   - `src/lib.rs`
   - `src/agent.rs`
   - `src/tracker.rs`
   - `src/template.rs`
   - `src/loader.rs`
   - `src/manifest.rs`

2. **`crates/tachikoma-cli/src/commands/plugins.rs`** - Plugin management

3. **Default plugins in repo:**
   - `.tachikoma/plugins/agents/claude/`
   - `.tachikoma/plugins/trackers/specs/`
   - `.tachikoma/plugins/templates/default/`

## Testing

- Test plugin discovery finds all plugins
- Test agent plugin execution
- Test tracker plugin state management
- Test Handlebars template rendering
- Test plugin validation catches errors
- Test hot-reload of templates

## References

- Ralph TUI plugin architecture
- Existing backend abstraction: `crates/tachikoma-backends-core/`
- Handlebars Rust: https://docs.rs/handlebars
