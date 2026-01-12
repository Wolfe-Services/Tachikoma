# Spec 094: Man Page Generation

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 094
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 090-cli-help
- **Estimated Context**: ~8%

## Objective

Implement man page generation for the CLI, producing properly formatted manual pages that integrate with system documentation.

## Acceptance Criteria

- [x] Generate man pages for main command
- [x] Generate man pages for all subcommands
- [x] Standard man page sections
- [x] Examples section
- [x] See Also cross-references
- [x] Output to file or stdout
- [x] Installation instructions
- [x] Proper roff formatting

## Implementation Details

### src/commands/manpages.rs

```rust
//! Man page generation command.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Command, CommandFactory};
use clap_mangen::Man;

use crate::cli::Cli;
use crate::error::CliError;

/// Generate man pages
#[derive(Debug, clap::Args)]
pub struct ManpagesCommand {
    /// Output directory for man pages
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// Generate for specific command only
    #[arg(short, long)]
    pub command: Option<String>,

    /// Print installation instructions
    #[arg(long)]
    pub install: bool,

    /// Output to stdout instead of files
    #[arg(long)]
    pub stdout: bool,
}

impl ManpagesCommand {
    pub fn execute(&self) -> Result<(), CliError> {
        if self.install {
            self.print_install_instructions();
            return Ok(());
        }

        let cmd = Cli::command();

        if self.stdout {
            self.generate_to_stdout(&cmd)?;
        } else {
            fs::create_dir_all(&self.output)?;
            self.generate_to_files(&cmd)?;
        }

        Ok(())
    }

    fn generate_to_stdout(&self, cmd: &Command) -> Result<(), CliError> {
        match &self.command {
            Some(name) => {
                let subcmd = find_subcommand(cmd, name)
                    .ok_or_else(|| CliError::not_found("command", name))?;
                self.render_man_page(subcmd, &mut io::stdout())?;
            }
            None => {
                self.render_man_page(cmd, &mut io::stdout())?;
            }
        }
        Ok(())
    }

    fn generate_to_files(&self, cmd: &Command) -> Result<(), CliError> {
        // Generate main command man page
        let path = self.output.join(format!("{}.1", cmd.get_name()));
        let mut file = fs::File::create(&path)?;
        self.render_man_page(cmd, &mut file)?;
        println!("Generated: {}", path.display());

        // Generate subcommand man pages
        for subcmd in cmd.get_subcommands() {
            if subcmd.is_hide_set() {
                continue;
            }

            let name = format!("{}-{}", cmd.get_name(), subcmd.get_name());
            let path = self.output.join(format!("{name}.1"));
            let mut file = fs::File::create(&path)?;
            self.render_subcommand_man_page(cmd.get_name(), subcmd, &mut file)?;
            println!("Generated: {}", path.display());

            // Generate nested subcommand man pages
            for nested in subcmd.get_subcommands() {
                if nested.is_hide_set() {
                    continue;
                }

                let name = format!(
                    "{}-{}-{}",
                    cmd.get_name(),
                    subcmd.get_name(),
                    nested.get_name()
                );
                let path = self.output.join(format!("{name}.1"));
                let mut file = fs::File::create(&path)?;
                self.render_nested_man_page(
                    cmd.get_name(),
                    subcmd.get_name(),
                    nested,
                    &mut file,
                )?;
                println!("Generated: {}", path.display());
            }
        }

        println!("\nGenerated man pages in: {}", self.output.display());
        Ok(())
    }

    fn render_man_page<W: Write>(&self, cmd: &Command, writer: &mut W) -> Result<(), CliError> {
        let man = Man::new(cmd.clone());
        man.render(writer)?;

        // Add custom sections
        self.render_examples(cmd, writer)?;
        self.render_environment(writer)?;
        self.render_files(writer)?;
        self.render_see_also(cmd, writer)?;

        Ok(())
    }

    fn render_subcommand_man_page<W: Write>(
        &self,
        parent: &str,
        cmd: &Command,
        writer: &mut W,
    ) -> Result<(), CliError> {
        // Custom header for subcommand
        writeln!(
            writer,
            r#".TH "{parent}-{name}" "1" "" "Tachikoma {version}" "Tachikoma Manual""#,
            name = cmd.get_name().to_uppercase(),
            version = env!("CARGO_PKG_VERSION"),
        )?;

        // NAME section
        writeln!(writer, ".SH NAME")?;
        writeln!(
            writer,
            "{parent}-{name} \\- {}",
            name = cmd.get_name(),
            cmd.get_about().map(|s| s.to_string()).unwrap_or_default()
        )?;

        // SYNOPSIS
        writeln!(writer, ".SH SYNOPSIS")?;
        writeln!(writer, ".B {parent} {name}", name = cmd.get_name())?;
        writeln!(writer, "[OPTIONS] [ARGS]")?;

        // DESCRIPTION
        if let Some(long_about) = cmd.get_long_about() {
            writeln!(writer, ".SH DESCRIPTION")?;
            writeln!(writer, "{long_about}")?;
        }

        // OPTIONS
        self.render_options(cmd, writer)?;

        // Custom sections
        self.render_examples(cmd, writer)?;
        self.render_see_also_subcommand(parent, cmd, writer)?;

        Ok(())
    }

    fn render_nested_man_page<W: Write>(
        &self,
        root: &str,
        parent: &str,
        cmd: &Command,
        writer: &mut W,
    ) -> Result<(), CliError> {
        let full_name = format!("{root}-{parent}-{}", cmd.get_name());

        writeln!(
            writer,
            r#".TH "{}" "1" "" "Tachikoma {}" "Tachikoma Manual""#,
            full_name.to_uppercase(),
            env!("CARGO_PKG_VERSION"),
        )?;

        // NAME
        writeln!(writer, ".SH NAME")?;
        writeln!(
            writer,
            "{full_name} \\- {}",
            cmd.get_about().map(|s| s.to_string()).unwrap_or_default()
        )?;

        // SYNOPSIS
        writeln!(writer, ".SH SYNOPSIS")?;
        writeln!(
            writer,
            ".B {root} {parent} {}",
            cmd.get_name()
        )?;
        writeln!(writer, "[OPTIONS] [ARGS]")?;

        // OPTIONS
        self.render_options(cmd, writer)?;

        // SEE ALSO
        writeln!(writer, ".SH SEE ALSO")?;
        writeln!(writer, ".BR {root} (1),")?;
        writeln!(writer, ".BR {root}-{parent} (1)")?;

        Ok(())
    }

    fn render_options<W: Write>(&self, cmd: &Command, writer: &mut W) -> Result<(), CliError> {
        let args: Vec<_> = cmd.get_arguments().collect();
        if args.is_empty() {
            return Ok(());
        }

        writeln!(writer, ".SH OPTIONS")?;

        for arg in args {
            if arg.is_positional() {
                continue;
            }

            let mut names = Vec::new();
            if let Some(short) = arg.get_short() {
                names.push(format!("\\-{short}"));
            }
            if let Some(long) = arg.get_long() {
                names.push(format!("\\-\\-{long}"));
            }

            writeln!(writer, ".TP")?;
            writeln!(writer, ".B {}", names.join(", "))?;

            if let Some(help) = arg.get_help() {
                writeln!(writer, "{help}")?;
            }
        }

        Ok(())
    }

    fn render_examples<W: Write>(&self, cmd: &Command, writer: &mut W) -> Result<(), CliError> {
        let examples = get_examples_for_command(cmd.get_name());
        if examples.is_empty() {
            return Ok(());
        }

        writeln!(writer, ".SH EXAMPLES")?;

        for (description, command) in examples {
            writeln!(writer, ".TP")?;
            writeln!(writer, "{description}")?;
            writeln!(writer, ".nf")?;
            writeln!(writer, "$ {command}")?;
            writeln!(writer, ".fi")?;
        }

        Ok(())
    }

    fn render_environment<W: Write>(&self, writer: &mut W) -> Result<(), CliError> {
        writeln!(writer, ".SH ENVIRONMENT")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B TACHIKOMA_CONFIG")?;
        writeln!(writer, "Path to the configuration file.")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B ANTHROPIC_API_KEY")?;
        writeln!(writer, "API key for Anthropic backend.")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B OPENAI_API_KEY")?;
        writeln!(writer, "API key for OpenAI backend.")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B NO_COLOR")?;
        writeln!(writer, "Disable color output when set.")?;

        Ok(())
    }

    fn render_files<W: Write>(&self, writer: &mut W) -> Result<(), CliError> {
        writeln!(writer, ".SH FILES")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B ~/.config/tachikoma/config.toml")?;
        writeln!(writer, "User configuration file.")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B ./tachikoma.toml")?;
        writeln!(writer, "Project configuration file.")?;

        writeln!(writer, ".TP")?;
        writeln!(writer, ".B ~/.cache/tachikoma/")?;
        writeln!(writer, "Cache directory for templates and tools.")?;

        Ok(())
    }

    fn render_see_also<W: Write>(&self, cmd: &Command, writer: &mut W) -> Result<(), CliError> {
        writeln!(writer, ".SH SEE ALSO")?;

        let subcommands: Vec<_> = cmd
            .get_subcommands()
            .filter(|c| !c.is_hide_set())
            .map(|c| format!(".BR {}-{} (1)", cmd.get_name(), c.get_name()))
            .collect();

        writeln!(writer, "{}", subcommands.join(",\n"))?;

        Ok(())
    }

    fn render_see_also_subcommand<W: Write>(
        &self,
        parent: &str,
        cmd: &Command,
        writer: &mut W,
    ) -> Result<(), CliError> {
        writeln!(writer, ".SH SEE ALSO")?;
        writeln!(writer, ".BR {parent} (1)")?;

        for subcmd in cmd.get_subcommands() {
            if !subcmd.is_hide_set() {
                writeln!(
                    writer,
                    ".BR {parent}-{}-{} (1)",
                    cmd.get_name(),
                    subcmd.get_name()
                )?;
            }
        }

        Ok(())
    }

    fn print_install_instructions(&self) {
        println!("Man Page Installation Instructions");
        println!("===================================\n");

        println!("1. Generate man pages:");
        println!("   tachikoma manpages --output /tmp/tachikoma-man\n");

        println!("2. Copy to system man directory:");
        println!("   sudo cp /tmp/tachikoma-man/*.1 /usr/local/share/man/man1/\n");

        println!("3. Update man database:");
        println!("   sudo mandb  # Linux");
        println!("   # or");
        println!("   sudo /usr/libexec/makewhatis /usr/local/share/man  # macOS\n");

        println!("4. View man page:");
        println!("   man tachikoma\n");

        println!("Alternative: Use MANPATH");
        println!("   export MANPATH=\"/path/to/tachikoma-man:$MANPATH\"");
        println!("   man tachikoma");
    }
}

fn find_subcommand<'a>(cmd: &'a Command, name: &str) -> Option<&'a Command> {
    for subcmd in cmd.get_subcommands() {
        if subcmd.get_name() == name {
            return Some(subcmd);
        }
        if let Some(found) = find_subcommand(subcmd, name) {
            return Some(found);
        }
    }
    None
}

fn get_examples_for_command(name: &str) -> Vec<(&'static str, &'static str)> {
    match name {
        "tachikoma" => vec![
            ("Initialize a new project", "tachikoma init my-project"),
            ("Check system health", "tachikoma doctor"),
            ("List configured backends", "tachikoma backends list"),
        ],
        "init" => vec![
            ("Create basic project", "tachikoma init my-agent"),
            ("Create with tools template", "tachikoma init my-agent --template tools"),
            ("Create in specific directory", "tachikoma init my-agent --path ~/projects"),
        ],
        "doctor" => vec![
            ("Run all checks", "tachikoma doctor"),
            ("Run backend checks only", "tachikoma doctor --category backends"),
            ("Output as JSON", "tachikoma --format json doctor"),
        ],
        "config" => vec![
            ("List all config", "tachikoma config list"),
            ("Get specific value", "tachikoma config get backend.default"),
            ("Set a value", "tachikoma config set agent.temperature 0.8"),
        ],
        "tools" => vec![
            ("List installed tools", "tachikoma tools list"),
            ("Install a tool", "tachikoma tools install filesystem"),
            ("Test a tool", "tachikoma tools test filesystem --input '{}'"),
        ],
        "backends" => vec![
            ("List backends", "tachikoma backends list"),
            ("Add Anthropic", "tachikoma backends add claude --backend-type anthropic"),
            ("Test connection", "tachikoma backends test"),
        ],
        _ => vec![],
    }
}
```

### Man Page Output Example

```roff
.TH "TACHIKOMA" "1" "" "Tachikoma 0.1.0" "Tachikoma Manual"
.SH NAME
tachikoma \- AI Agent Development Framework
.SH SYNOPSIS
.B tachikoma
[\fIOPTIONS\fR] <\fICOMMAND\fR>
.SH DESCRIPTION
Tachikoma is a framework for building, testing, and deploying AI agents
with MCP (Model Context Protocol) integration.
.SH OPTIONS
.TP
.B \-v, \-\-verbose
Increase verbosity level. Can be specified multiple times.
.TP
.B \-q, \-\-quiet
Suppress all output except errors.
.TP
.B \-c, \-\-config \fIFILE\fR
Path to configuration file.
.TP
.B \-\-color \fIMODE\fR
When to use colors: auto, always, never.
.TP
.B \-\-format \fIFORMAT\fR
Output format: text, json.
.SH COMMANDS
.TP
.B init
Initialize a new Tachikoma project.
.TP
.B config
Manage configuration.
.TP
.B doctor
Check system health and dependencies.
.TP
.B tools
Manage MCP tools.
.TP
.B backends
Manage AI backends.
.SH EXAMPLES
.TP
Initialize a new project
.nf
$ tachikoma init my-project
.fi
.TP
Check system health
.nf
$ tachikoma doctor
.fi
.TP
List configured backends
.nf
$ tachikoma backends list
.fi
.SH ENVIRONMENT
.TP
.B TACHIKOMA_CONFIG
Path to the configuration file.
.TP
.B ANTHROPIC_API_KEY
API key for Anthropic backend.
.TP
.B OPENAI_API_KEY
API key for OpenAI backend.
.SH FILES
.TP
.B ~/.config/tachikoma/config.toml
User configuration file.
.TP
.B ./tachikoma.toml
Project configuration file.
.SH SEE ALSO
.BR tachikoma-init (1),
.BR tachikoma-config (1),
.BR tachikoma-doctor (1),
.BR tachikoma-tools (1),
.BR tachikoma-backends (1)
.SH BUGS
Report bugs at: https://github.com/tachikoma/tachikoma/issues
.SH AUTHORS
Tachikoma Contributors
```

## Testing Requirements

### Integration Tests

```rust
// tests/manpages.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_manpages_generation() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["manpages", "--output", dir.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(dir.path().join("tachikoma.1").exists());
}

#[test]
fn test_manpages_stdout() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["manpages", "--stdout"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH"));
}

#[test]
fn test_manpages_specific_command() {
    Command::cargo_bin("tachikoma")
        .unwrap()
        .args(["manpages", "--stdout", "--command", "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"));
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **090-cli-help.md**: Help system
- **093-cli-completions.md**: Shell completions
