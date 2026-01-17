# Tachikoma Specs

Welcome to your Tachikoma project! This directory contains specifications that define what your AI agents should build.

## What is a Spec?

A spec is a markdown file that describes:
- **What** to build (requirements, features)
- **Acceptance criteria** (checkboxes that get marked complete)
- **Context** (background, constraints, references)

## File Naming Convention

```
001-getting-started.md      # Simple tasks first
002-setup-database.md       # Build incrementally  
003-user-authentication.md  # Each spec = one mission
```

## Spec Structure

```markdown
# Spec XXX: Your Feature Name

**Status:** Ready | In Progress | Complete
**Priority:** P0 (Critical) | P1 (High) | P2 (Medium) | P3 (Low)

## Overview
Brief description of what this accomplishes.

## Acceptance Criteria
- [ ] First thing to implement
- [ ] Second thing to implement  
- [ ] Tests pass
- [ ] Documentation updated

## Implementation Details
Technical guidance, examples, patterns to follow.
```

## Getting Started

1. **Read** `001-getting-started.md` 
2. **Run** `tachikoma run --spec 001`
3. **Watch** Tachikoma implement it step by step
4. **Create** your own specs with `tachikoma chat`

## Best Practices

- **One mission per spec** - Keep scope focused
- **Clear acceptance criteria** - Tachikoma marks checkboxes as it works
- **Test everything** - Include testing in your criteria
- **Build incrementally** - Start simple, add complexity

Happy coding with your AI team! 🕷️