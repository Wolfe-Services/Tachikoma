# Spec 093: Shell Completions

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 093
- **Status**: Planned
- **Dependencies**: 076-cli-crate
- **Estimated Context**: ~8%

## Objective

Implement shell completion generation for the CLI, supporting bash, zsh, fish, PowerShell, and elvish shells with dynamic completions for commands, options, and values.

## Acceptance Criteria

- [x] Generate static completions for all shells
- [x] Dynamic completions for tool names
- [x] Dynamic completions for backend names
- [x] Dynamic completions for config keys
- [x] File path completions where appropriate
- [x] Hidden command from help
- [x] Installation instructions
- [x] Completion scripts output to stdout or file

## Implementation Details

### src/commands/completions.rs

```rust
//! Shell completion generation.

use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Command, CommandFactory};
use clap_complete::{generate, Shell};

use crate::cli::Cli;
use crate::error::CliError;

/// Generate shell completions
#[derive(Debug, clap::Args)]
pub struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Print installation instructions
    #[arg(long)]
    pub install: bool,
}

impl CompletionsCommand {
    pub fn execute(&self) -> Result<(), CliError> {
        if self.install {
            self.print_install_instructions();
            return Ok(());
        }

        let mut cmd = Cli::command();

        // Add dynamic completers
        self.add_dynamic_completers(&mut cmd);

        match &self.output {
            Some(path) => {
                let mut file = std::fs::File::create(path)?;
                generate(self.shell, &mut cmd, "tachikoma", &mut file);
                println!("Completions written to: {}", path.display());
            }
            None => {
                generate(self.shell, &mut cmd, "tachikoma", &mut io::stdout());
            }
        }

        Ok(())
    }

    fn add_dynamic_completers(&self, _cmd: &mut Command) {
        // Dynamic completers would be added here
        // This is limited by clap_complete's static generation
    }

    fn print_install_instructions(&self) {
        match self.shell {
            Shell::Bash => {
                println!("# Add to ~/.bashrc or ~/.bash_profile:");
                println!();
                println!("# Option 1: Source directly (slower startup)");
                println!("source <(tachikoma completions bash)");
                println!();
                println!("# Option 2: Generate and cache (recommended)");
                println!("tachikoma completions bash > ~/.local/share/bash-completion/completions/tachikoma");
            }
            Shell::Zsh => {
                println!("# Add to ~/.zshrc:");
                println!();
                println!("# Ensure completion system is enabled");
                println!("autoload -Uz compinit && compinit");
                println!();
                println!("# Option 1: Source directly");
                println!("source <(tachikoma completions zsh)");
                println!();
                println!("# Option 2: Add to fpath (recommended)");
                println!("# First generate the completion file:");
                println!("tachikoma completions zsh > ~/.zfunc/_tachikoma");
                println!();
                println!("# Then add to fpath in ~/.zshrc (before compinit):");
                println!("fpath=(~/.zfunc $fpath)");
            }
            Shell::Fish => {
                println!("# Generate and install:");
                println!("tachikoma completions fish > ~/.config/fish/completions/tachikoma.fish");
                println!();
                println!("# Or source directly in config.fish:");
                println!("tachikoma completions fish | source");
            }
            Shell::PowerShell => {
                println!("# Add to your PowerShell profile:");
                println!("# (typically at $PROFILE)");
                println!();
                println!("Invoke-Expression (tachikoma completions powershell | Out-String)");
                println!();
                println!("# Or save to a file and dot-source it:");
                println!("tachikoma completions powershell > tachikoma.ps1");
                println!(". ./tachikoma.ps1");
            }
            Shell::Elvish => {
                println!("# Add to ~/.elvish/rc.elv:");
                println!();
                println!("eval (tachikoma completions elvish | slurp)");
            }
            _ => {
                println!("Installation instructions not available for this shell.");
                println!("Generate completions with: tachikoma completions {}", self.shell);
            }
        }
    }
}

/// Custom completion script for dynamic values
pub fn generate_dynamic_bash_completions() -> String {
    r#"
# Dynamic completions for tachikoma

_tachikoma_dynamic_completions() {
    local cur prev words cword
    _init_completion || return

    case "${prev}" in
        tools)
            case "${words[2]}" in
                show|test|enable|disable|uninstall)
                    # Complete with installed tool names
                    local tools=$(tachikoma tools list --format=names 2>/dev/null)
                    COMPREPLY=($(compgen -W "${tools}" -- "${cur}"))
                    return 0
                    ;;
            esac
            ;;
        backends)
            case "${words[2]}" in
                show|test|remove|default|configure)
                    # Complete with configured backend names
                    local backends=$(tachikoma backends list --format=names 2>/dev/null)
                    COMPREPLY=($(compgen -W "${backends}" -- "${cur}"))
                    return 0
                    ;;
            esac
            ;;
        config)
            case "${words[2]}" in
                get|set)
                    # Complete with config keys
                    local keys=$(tachikoma config list --format=keys 2>/dev/null)
                    COMPREPLY=($(compgen -W "${keys}" -- "${cur}"))
                    return 0
                    ;;
            esac
            ;;
    esac

    return 1
}

# Hook into the main completion function
_tachikoma_completions() {
    # Try dynamic completions first
    _tachikoma_dynamic_completions && return 0

    # Fall back to static completions
    _tachikoma
}

complete -F _tachikoma_completions tachikoma
"#.to_string()
}

/// Custom completion script for zsh with dynamic values
pub fn generate_dynamic_zsh_completions() -> String {
    r#"
#compdef tachikoma

# Dynamic completions for tachikoma

_tachikoma_tools() {
    local tools
    tools=(${(f)"$(tachikoma tools list --format=names 2>/dev/null)"})
    _describe -t tools 'tool' tools
}

_tachikoma_backends() {
    local backends
    backends=(${(f)"$(tachikoma backends list --format=names 2>/dev/null)"})
    _describe -t backends 'backend' backends
}

_tachikoma_config_keys() {
    local keys
    keys=(${(f)"$(tachikoma config list --format=keys 2>/dev/null)"})
    _describe -t keys 'config key' keys
}

_tachikoma_templates() {
    local templates
    templates=(basic tools workflow chat minimal)
    _describe -t templates 'template' templates
}

_tachikoma() {
    local line state

    _arguments -C \
        '(-v --verbose)'{-v,--verbose}'[Increase verbosity]' \
        '(-q --quiet)'{-q,--quiet}'[Suppress output]' \
        '(-c --config)'{-c,--config}'[Config file]:file:_files' \
        '--color[Color mode]:mode:(auto always never)' \
        '--format[Output format]:format:(text json)' \
        '1: :->command' \
        '*::arg:->args'

    case "$state" in
        command)
            local -a commands
            commands=(
                'init:Initialize a new project'
                'config:Manage configuration'
                'doctor:Check system health'
                'tools:Manage MCP tools'
                'backends:Manage AI backends'
            )
            _describe -t commands 'command' commands
            ;;
        args)
            case "$line[1]" in
                tools)
                    _arguments -C \
                        '1: :->subcommand' \
                        '*::arg:->tool_args'
                    case "$state" in
                        subcommand)
                            local -a subcommands
                            subcommands=(
                                'list:List tools'
                                'show:Show tool details'
                                'install:Install a tool'
                                'uninstall:Uninstall a tool'
                                'test:Test a tool'
                                'enable:Enable a tool'
                                'disable:Disable a tool'
                            )
                            _describe -t subcommands 'subcommand' subcommands
                            ;;
                        tool_args)
                            case "$line[2]" in
                                show|test|enable|disable|uninstall)
                                    _tachikoma_tools
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                backends)
                    _arguments -C \
                        '1: :->subcommand' \
                        '*::arg:->backend_args'
                    case "$state" in
                        subcommand)
                            local -a subcommands
                            subcommands=(
                                'list:List backends'
                                'add:Add a backend'
                                'remove:Remove a backend'
                                'show:Show backend details'
                                'test:Test backend connectivity'
                                'default:Set default backend'
                            )
                            _describe -t subcommands 'subcommand' subcommands
                            ;;
                        backend_args)
                            case "$line[2]" in
                                show|remove|test|default)
                                    _tachikoma_backends
                                    ;;
                                add)
                                    _arguments \
                                        '(-t --backend-type)'{-t,--backend-type}'[Backend type]:type:(anthropic openai ollama local)' \
                                        '--api-key[API key]:key:' \
                                        '--base-url[Base URL]:url:' \
                                        '(-d --default)'{-d,--default}'[Set as default]'
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                config)
                    _arguments -C \
                        '1: :->subcommand' \
                        '*::arg:->config_args'
                    case "$state" in
                        subcommand)
                            local -a subcommands
                            subcommands=(
                                'get:Get a value'
                                'set:Set a value'
                                'list:List all'
                                'edit:Edit config'
                                'path:Show config path'
                                'init:Initialize config'
                            )
                            _describe -t subcommands 'subcommand' subcommands
                            ;;
                        config_args)
                            case "$line[2]" in
                                get|set)
                                    _tachikoma_config_keys
                                    ;;
                            esac
                            ;;
                    esac
                    ;;
                init)
                    _arguments \
                        '1:name:' \
                        '(-t --template)'{-t,--template}'[Template]:template:_tachikoma_templates' \
                        '(-p --path)'{-p,--path}'[Directory]:dir:_files -/' \
                        '--no-prompt[Skip prompts]' \
                        '--no-git[Skip git init]' \
                        '(-f --force)'{-f,--force}'[Force overwrite]'
                    ;;
            esac
            ;;
    esac
}

_tachikoma "$@"
"#.to_string()
}

/// Generate fish completions with dynamic values
pub fn generate_dynamic_fish_completions() -> String {
    r#"
# Fish completions for tachikoma

# Disable file completions by default
complete -c tachikoma -f

# Helper functions for dynamic completions
function __tachikoma_tools
    tachikoma tools list --format=names 2>/dev/null
end

function __tachikoma_backends
    tachikoma backends list --format=names 2>/dev/null
end

function __tachikoma_config_keys
    tachikoma config list --format=keys 2>/dev/null
end

# Global options
complete -c tachikoma -s v -l verbose -d 'Increase verbosity'
complete -c tachikoma -s q -l quiet -d 'Suppress output'
complete -c tachikoma -s c -l config -d 'Config file' -r -F
complete -c tachikoma -l color -d 'Color mode' -xa 'auto always never'
complete -c tachikoma -l format -d 'Output format' -xa 'text json'

# Subcommands
complete -c tachikoma -n __fish_use_subcommand -a init -d 'Initialize a new project'
complete -c tachikoma -n __fish_use_subcommand -a config -d 'Manage configuration'
complete -c tachikoma -n __fish_use_subcommand -a doctor -d 'Check system health'
complete -c tachikoma -n __fish_use_subcommand -a tools -d 'Manage MCP tools'
complete -c tachikoma -n __fish_use_subcommand -a backends -d 'Manage AI backends'

# init options
complete -c tachikoma -n '__fish_seen_subcommand_from init' -s t -l template -d 'Template' -xa 'basic tools workflow chat minimal'
complete -c tachikoma -n '__fish_seen_subcommand_from init' -s p -l path -d 'Directory' -r -a '(__fish_complete_directories)'
complete -c tachikoma -n '__fish_seen_subcommand_from init' -l no-prompt -d 'Skip prompts'
complete -c tachikoma -n '__fish_seen_subcommand_from init' -s f -l force -d 'Force overwrite'

# tools subcommands
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a list -d 'List tools'
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a show -d 'Show tool details'
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a install -d 'Install a tool'
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a uninstall -d 'Uninstall a tool'
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a test -d 'Test a tool'
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a enable -d 'Enable a tool'
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and not __fish_seen_subcommand_from list show install uninstall test enable disable' -a disable -d 'Disable a tool'

# Dynamic tool name completion
complete -c tachikoma -n '__fish_seen_subcommand_from tools; and __fish_seen_subcommand_from show test enable disable uninstall' -xa '(__tachikoma_tools)'

# backends subcommands
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and not __fish_seen_subcommand_from list add remove show test default' -a list -d 'List backends'
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and not __fish_seen_subcommand_from list add remove show test default' -a add -d 'Add a backend'
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and not __fish_seen_subcommand_from list add remove show test default' -a remove -d 'Remove a backend'
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and not __fish_seen_subcommand_from list add remove show test default' -a show -d 'Show backend details'
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and not __fish_seen_subcommand_from list add remove show test default' -a test -d 'Test connectivity'
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and not __fish_seen_subcommand_from list add remove show test default' -a default -d 'Set default backend'

# Dynamic backend name completion
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and __fish_seen_subcommand_from show remove test default' -xa '(__tachikoma_backends)'

# backends add options
complete -c tachikoma -n '__fish_seen_subcommand_from backends; and __fish_seen_subcommand_from add' -s t -l backend-type -d 'Backend type' -xa 'anthropic openai ollama local'

# config subcommands
complete -c tachikoma -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from get set list edit path init' -a get -d 'Get a value'
complete -c tachikoma -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from get set list edit path init' -a set -d 'Set a value'
complete -c tachikoma -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from get set list edit path init' -a list -d 'List all'
complete -c tachikoma -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from get set list edit path init' -a edit -d 'Edit config'
complete -c tachikoma -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from get set list edit path init' -a path -d 'Show config path'
complete -c tachikoma -n '__fish_seen_subcommand_from config; and not __fish_seen_subcommand_from get set list edit path init' -a init -d 'Initialize config'

# Dynamic config key completion
complete -c tachikoma -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from get set' -xa '(__tachikoma_config_keys)'
"#.to_string()
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bash_completions() {
        let output = generate_dynamic_bash_completions();
        assert!(output.contains("_tachikoma"));
        assert!(output.contains("tools"));
        assert!(output.contains("backends"));
    }

    #[test]
    fn test_generate_zsh_completions() {
        let output = generate_dynamic_zsh_completions();
        assert!(output.contains("#compdef tachikoma"));
        assert!(output.contains("_tachikoma_tools"));
        assert!(output.contains("_tachikoma_backends"));
    }

    #[test]
    fn test_generate_fish_completions() {
        let output = generate_dynamic_fish_completions();
        assert!(output.contains("complete -c tachikoma"));
        assert!(output.contains("__tachikoma_tools"));
    }
}
```

### Integration Tests

```rust
// tests/completions.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_completions_bash() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tachikoma"));
}

#[test]
fn test_completions_zsh() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef"));
}

#[test]
fn test_completions_fish() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **077-cli-args.md**: Argument definitions
- **090-cli-help.md**: Help system
