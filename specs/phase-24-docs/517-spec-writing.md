# Spec 517: Spec Writing Guide

## Overview
Guide for writing effective Tachikoma specification files, covering syntax, best practices, patterns, and anti-patterns.

## Requirements

### Spec File Anatomy
- File header and metadata
- Overview section purpose
- Requirements structure
- Dependencies declaration
- Verification criteria

### Metadata Fields
```yaml
---
id: 517
title: Spec Writing Guide
phase: 24
category: documentation
status: draft
priority: medium
dependencies: [511]
tags: [docs, guide, meta]
---
```

### Writing Requirements
- Use clear, actionable language
- One requirement per bullet point
- Specify acceptance criteria
- Include measurable outcomes
- Avoid ambiguous terms

### Section Templates
- Overview: What and why (2-3 sentences)
- Requirements: Specific needs (bullet lists)
- Implementation: Technical details
- Dependencies: Related specs
- Verification: Test criteria

### Common Patterns
- Feature specification pattern
- Integration specification pattern
- Refactoring specification pattern
- Bug fix specification pattern
- Research specification pattern

### Anti-Patterns to Avoid
- Vague requirements ("make it better")
- Missing acceptance criteria
- Circular dependencies
- Over-specification
- Under-specification

### Spec Lifecycle
- Draft: Initial creation
- Review: Peer review phase
- Approved: Ready for implementation
- In Progress: Being implemented
- Complete: Fully implemented
- Deprecated: No longer relevant

### Quality Checklist
- [ ] Clear problem statement
- [ ] Specific requirements
- [ ] Testable criteria
- [ ] Dependencies identified
- [ ] No ambiguous language
- [ ] Appropriate scope

## Dependencies
- Spec 511: Documentation Structure

## Verification
- [ ] Template provided
- [ ] Examples included
- [ ] Anti-patterns documented
- [ ] Lifecycle explained
- [ ] Checklist usable
