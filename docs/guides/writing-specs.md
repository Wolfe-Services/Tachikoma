# Writing Effective Specifications

This guide explains how to write clear, actionable specifications for Tachikoma.

## Specification Structure

Each spec follows a consistent structure:

```markdown
# [Number] - [Title]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [3-digit number]
**Status:** [Planned|In Progress|Complete]
**Dependencies:** [List of dependent specs]

## Objective
Clear statement of what this spec achieves.

## Acceptance Criteria
- [ ] Specific, testable criteria
- [ ] Each criterion should be independently verifiable

## Implementation Details
Technical details and code examples.

## Testing Requirements
How to verify the implementation works.
```

## Best Practices

1. **Be Specific**: Each criterion should be unambiguous
2. **Be Testable**: Include concrete testing requirements
3. **Be Atomic**: Each spec should focus on one cohesive feature
4. **Include Examples**: Show expected code/configuration

## Phase Organization

Specs are organized into phases that build on each other:
- **Phase 0**: Project setup and tooling
- **Phase 1**: Core types and utilities
- **Phase 2+**: Feature implementation