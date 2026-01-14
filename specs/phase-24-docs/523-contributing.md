# Spec 523: Contributing Guide

## Overview
Comprehensive guide for contributors covering code contributions, documentation, testing, and community participation.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Getting Started
- Repository setup
- Development environment
- Building from source
- Running tests locally
- Code style overview

### Contribution Types
- Bug reports
- Feature requests
- Code contributions
- Documentation improvements
- Translation help
- Community support

### Code Contribution Process
1. Fork the repository
2. Create feature branch
3. Make changes
4. Write/update tests
5. Run linting and tests
6. Submit pull request
7. Address review feedback
8. Merge upon approval

### Commit Message Format
```
type(scope): subject

body

footer
```
- Types: feat, fix, docs, style, refactor, test, chore
- Scope: component affected
- Subject: imperative mood, max 50 chars

### Pull Request Template
```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing done

## Checklist
- [ ] Code follows style guide
- [ ] Self-reviewed
- [ ] Comments added
- [ ] Docs updated
```

### Code Review Guidelines
- Constructive feedback
- Focus on code, not person
- Suggest improvements
- Approve when ready
- Request changes clearly

### Development Setup
```bash
# Clone repository
git clone https://github.com/org/tachikoma
cd tachikoma

# Install dependencies
make deps

# Run tests
make test

# Build binary
make build
```

### Community Guidelines
- Code of Conduct adherence
- Respectful communication
- Inclusive language
- Recognition of contributions

## Dependencies
- Spec 511: Documentation Structure

## Verification
- [ ] Setup instructions work
- [ ] Templates provided
- [ ] Process documented
- [ ] Guidelines clear
- [ ] CoC linked
