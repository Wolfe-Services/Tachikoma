# Spec 521: Example Projects

## Overview
Collection of example projects demonstrating Tachikoma usage across different scenarios, languages, and complexity levels.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Example Categories
- Starter examples (minimal setup)
- Language-specific examples
- Integration examples
- Advanced patterns
- Real-world scenarios

### Starter Examples
- hello-world: Minimal agent configuration
- simple-task: Basic task execution
- multi-step: Sequential task workflow
- scheduled: Cron-based execution
- conditional: Conditional task logic

### Language Examples
- go-project: Go development workflow
- python-project: Python/pip workflow
- node-project: Node.js/npm workflow
- rust-project: Rust/cargo workflow
- multi-lang: Polyglot repository

### Integration Examples
- github-actions: CI/CD integration
- docker-compose: Container orchestration
- kubernetes: K8s deployment
- terraform: Infrastructure as code
- database-migrations: Schema management

### Advanced Examples
- parallel-execution: Concurrent tasks
- dependency-graph: Complex dependencies
- custom-hooks: Event-driven workflows
- api-integration: External API calls
- self-modifying: Dynamic spec generation

### Example Project Structure
```
examples/
├── README.md
├── starter/
│   ├── hello-world/
│   │   ├── README.md
│   │   ├── tachikoma.yaml
│   │   └── specs/
│   └── ...
├── languages/
├── integrations/
└── advanced/
```

### Example Requirements
- README with description
- Working configuration
- Clear instructions
- Expected output documented
- Automated testing

### Maintenance
- CI testing of all examples
- Version compatibility notes
- Deprecation handling
- Community contributions

## Dependencies
- Spec 511: Documentation Structure
- Spec 514: User Guide

## Verification
- [ ] All examples run successfully
- [ ] READMEs complete
- [ ] CI tests pass
- [ ] Version compatibility noted
- [ ] Categories well-organized
