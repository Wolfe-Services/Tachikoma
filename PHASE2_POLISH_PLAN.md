# Phase 2: Polish Pass with ralph-tui + Beads

> Run this AFTER Phase 1 (Ralph loop) completes all specs.

## Overview

Use ralph-tui with Beads tracker for a code quality/polish pass after the app is functionally complete.

## Prerequisites

```bash
# Install bun if needed
curl -fsSL https://bun.sh/install | bash

# Install ralph-tui
bun install -g ralph-tui

# Setup in Tachikoma
cd /Users/mubix/Documents/code/Tachikoma
ralph-tui setup
```

## Create Polish Beads

```bash
cd /Users/mubix/Documents/code/Tachikoma

# Code Simplification
bd create --title="Simplify quality.rs (554 lines) - extract into modules" --type=task --priority=2
bd create --title="Reduce cyclomatic complexity in forge crate" --type=task --priority=2
bd create --title="Extract common UI patterns into shared components" --type=task --priority=2

# Dead Code Removal
bd create --title="Remove unused exports across all crates" --type=task --priority=2
bd create --title="Delete test placeholders and mock data" --type=task --priority=3
bd create --title="Clean up commented-out code blocks" --type=task --priority=3

# Type Safety
bd create --title="Replace 'any' types with proper TypeScript types" --type=task --priority=2
bd create --title="Add missing return types to functions" --type=task --priority=3
bd create --title="Tighten Rust type bounds where possible" --type=task --priority=3

# UI Polish
bd create --title="Consolidate 6 chart components into reusable ChartBase" --type=task --priority=2
bd create --title="Add loading skeletons to all async components" --type=task --priority=3
bd create --title="Ensure consistent error boundaries" --type=task --priority=2
bd create --title="Add keyboard navigation to all interactive elements" --type=task --priority=3

# Performance
bd create --title="Audit bundle size - remove unused dependencies" --type=task --priority=2
bd create --title="Add React.memo/Svelte reactivity optimization" --type=task --priority=3
bd create --title="Lazy load heavy components" --type=task --priority=3

# Documentation
bd create --title="Add JSDoc/rustdoc to public APIs" --type=task --priority=3
bd create --title="Update README with current architecture" --type=task --priority=3
```

## Run Polish Loop

```bash
# Create an epic to group polish tasks
bd create --title="Phase 2: Code Polish" --type=epic --priority=1

# Run ralph-tui with low iterations for careful review
ralph-tui run --epic phase-2-code-polish --max-iterations 5
```

## Code Simplifier Patterns

Based on Claude's internal code-simplifier (open-sourced):

1. **Chunk by complexity** - Target functions with high cyclomatic complexity
2. **Extract common patterns** - Find repeated code blocks and DRY them
3. **Dead code elimination** - Remove unused exports/functions
4. **Type tightening** - Replace loose types with strict ones

### Example Prompt for Simplification

```
Review this file and:
1. Identify functions over 30 lines - split them
2. Find repeated patterns - extract to helpers
3. Remove any dead code or unused imports
4. Improve type safety where possible

Do not change functionality. Focus only on readability and maintainability.
```

## Success Criteria

- [ ] No functions over 50 lines
- [ ] No duplicate code blocks > 10 lines
- [ ] Zero `any` types in TypeScript
- [ ] All public APIs documented
- [ ] Bundle size reduced by 20%+
- [ ] All components have error boundaries

## Reference

- [ralph-tui docs](https://ralph-tui.com/docs/getting-started/quick-start)
- [Claude Code Simplifier](https://www.reddit.com/r/ClaudeAI/comments/1q8h6oz/claude_code_creator_open_sources_the_internal/)
- [Beads Issue Tracker](https://github.com/anthropics/beads)
