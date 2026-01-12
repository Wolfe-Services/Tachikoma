# Spec 513: CLI Documentation

## Overview
Complete command-line interface documentation including all commands, flags, environment variables, and shell integration.

## Requirements

### Command Reference
- All commands with synopsis
- Flag descriptions and defaults
- Argument types and validation
- Subcommand hierarchy
- Global vs local flags

### Auto-Generated Docs
- Generated from cobra command definitions
- Man page generation (groff format)
- Markdown reference pages
- Shell completion scripts
- Command tree visualization

### Usage Examples
- Common workflows
- Piping and scripting examples
- JSON output processing with jq
- Integration with other tools
- Error handling patterns

### Environment Variables
- Complete variable reference
- Precedence rules (flag > env > config)
- Secret handling best practices
- Platform-specific paths

### Shell Integration
- Bash completion setup
- Zsh completion setup
- Fish completion setup
- PowerShell completion
- Alias recommendations

### Interactive Features
- TUI mode documentation
- Keyboard shortcuts
- Configuration wizard
- Progress indicators

## Generated Artifacts
```
docs/reference/cli/
├── tachikoma.md
├── tachikoma-agent.md
├── tachikoma-spec.md
├── tachikoma-task.md
├── tachikoma-config.md
├── environment.md
├── completion/
│   ├── bash.sh
│   ├── zsh.sh
│   ├── fish.fish
│   └── powershell.ps1
└── man/
    └── tachikoma.1
```

## Dependencies
- Spec 511: Documentation Structure

## Verification
- [ ] All commands documented
- [ ] Man pages generate correctly
- [ ] Completion scripts work
- [ ] Examples are executable
- [ ] Environment vars complete
