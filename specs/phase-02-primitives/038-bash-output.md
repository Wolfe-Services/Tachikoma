# 038 - Bash Output Capture

**Phase:** 2 - Five Primitives
**Spec ID:** 038
**Status:** Planned
**Dependencies:** 036-bash-exec-core
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement stdout/stderr capture with streaming support, output size limits, and real-time output handling.

---

## Acceptance Criteria

- [x] Separate stdout and stderr capture
- [x] Combined output option
- [x] Output size limits with truncation
- [x] Streaming output for long-running commands
- [x] Line-based output callbacks
- [x] Binary output handling

---

## Implementation Details

### 1. Output Capture Module (src/bash/output.rs)

```rust
//! Output capture for bash commands.

use std::io::Write;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::mpsc;
use tracing::debug;

/// Maximum output size (10 MB).
pub const DEFAULT_MAX_OUTPUT: usize = 10 * 1024 * 1024;

/// Captured output from a command.
#[derive(Debug, Clone)]
pub struct CapturedOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Whether stdout was truncated.
    pub stdout_truncated: bool,
    /// Whether stderr was truncated.
    pub stderr_truncated: bool,
    /// Total bytes received before truncation.
    pub stdout_total_bytes: usize,
    pub stderr_total_bytes: usize,
}

impl CapturedOutput {
    /// Create empty output.
    pub fn empty() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            stdout_total_bytes: 0,
            stderr_total_bytes: 0,
        }
    }

    /// Combine stdout and stderr.
    pub fn combined(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }

    /// Check if any output was truncated.
    pub fn is_truncated(&self) -> bool {
        self.stdout_truncated || self.stderr_truncated
    }
}

/// Configuration for output capture.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Maximum stdout size.
    pub max_stdout: usize,
    /// Maximum stderr size.
    pub max_stderr: usize,
    /// Whether to strip ANSI codes.
    pub strip_ansi: bool,
    /// Whether to trim whitespace.
    pub trim_output: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            max_stdout: DEFAULT_MAX_OUTPUT,
            max_stderr: DEFAULT_MAX_OUTPUT,
            strip_ansi: true,
            trim_output: true,
        }
    }
}

/// Capture stdout and stderr concurrently.
pub async fn capture_output(
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
    config: &OutputConfig,
) -> CapturedOutput {
    let stdout_future = capture_stream(stdout, config.max_stdout);
    let stderr_future = capture_stream_stderr(stderr, config.max_stderr);

    let ((stdout, stdout_truncated, stdout_total), (stderr, stderr_truncated, stderr_total)) =
        tokio::join!(stdout_future, stderr_future);

    let mut output = CapturedOutput {
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
        stdout_total_bytes: stdout_total,
        stderr_total_bytes: stderr_total,
    };

    // Post-process
    if config.strip_ansi {
        output.stdout = strip_ansi_codes(&output.stdout);
        output.stderr = strip_ansi_codes(&output.stderr);
    }

    if config.trim_output {
        output.stdout = output.stdout.trim().to_string();
        output.stderr = output.stderr.trim().to_string();
    }

    output
}

async fn capture_stream(
    stream: Option<ChildStdout>,
    max_size: usize,
) -> (String, bool, usize) {
    let Some(stream) = stream else {
        return (String::new(), false, 0);
    };

    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::with_capacity(max_size.min(1024 * 1024));
    let mut total = 0;
    let mut truncated = false;

    loop {
        let mut chunk = vec![0u8; 8192];
        match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                let remaining = max_size.saturating_sub(buffer.len());
                if remaining > 0 {
                    let to_add = n.min(remaining);
                    buffer.extend_from_slice(&chunk[..to_add]);
                    if to_add < n {
                        truncated = true;
                    }
                } else {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }

    (String::from_utf8_lossy(&buffer).into_owned(), truncated, total)
}

async fn capture_stream_stderr(
    stream: Option<ChildStderr>,
    max_size: usize,
) -> (String, bool, usize) {
    let Some(stream) = stream else {
        return (String::new(), false, 0);
    };

    let mut reader = BufReader::new(stream);
    let mut buffer = Vec::with_capacity(max_size.min(1024 * 1024));
    let mut total = 0;
    let mut truncated = false;

    loop {
        let mut chunk = vec![0u8; 8192];
        match reader.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                let remaining = max_size.saturating_sub(buffer.len());
                if remaining > 0 {
                    let to_add = n.min(remaining);
                    buffer.extend_from_slice(&chunk[..to_add]);
                    if to_add < n {
                        truncated = true;
                    }
                } else {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }

    (String::from_utf8_lossy(&buffer).into_owned(), truncated, total)
}

/// Strip ANSI escape codes from string.
pub fn strip_ansi_codes(s: &str) -> String {
    // Simple ANSI escape sequence pattern
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Skip until letter
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Output line from streaming.
#[derive(Debug, Clone)]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
}

/// Stream output line by line.
pub struct OutputStreamer {
    tx: mpsc::Sender<OutputLine>,
    rx: mpsc::Receiver<OutputLine>,
}

impl OutputStreamer {
    /// Create a new output streamer.
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self { tx, rx }
    }

    /// Get a sender for streaming.
    pub fn sender(&self) -> mpsc::Sender<OutputLine> {
        self.tx.clone()
    }

    /// Receive the next line.
    pub async fn recv(&mut self) -> Option<OutputLine> {
        self.rx.recv().await
    }

    /// Start streaming from stdout.
    pub async fn stream_stdout(
        stdout: ChildStdout,
        tx: mpsc::Sender<OutputLine>,
    ) {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(OutputLine::Stdout(line)).await.is_err() {
                break;
            }
        }
    }

    /// Start streaming from stderr.
    pub async fn stream_stderr(
        stderr: ChildStderr,
        tx: mpsc::Sender<OutputLine>,
    ) {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(OutputLine::Stderr(line)).await.is_err() {
                break;
            }
        }
    }
}

/// Callback for real-time output.
pub trait OutputCallback: Send + Sync {
    fn on_stdout(&self, line: &str);
    fn on_stderr(&self, line: &str);
}

/// A simple callback that collects output.
pub struct CollectingCallback {
    stdout: std::sync::Mutex<Vec<String>>,
    stderr: std::sync::Mutex<Vec<String>>,
}

impl CollectingCallback {
    pub fn new() -> Self {
        Self {
            stdout: std::sync::Mutex::new(Vec::new()),
            stderr: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn stdout(&self) -> Vec<String> {
        self.stdout.lock().unwrap().clone()
    }

    pub fn stderr(&self) -> Vec<String> {
        self.stderr.lock().unwrap().clone()
    }
}

impl OutputCallback for CollectingCallback {
    fn on_stdout(&self, line: &str) {
        self.stdout.lock().unwrap().push(line.to_string());
    }

    fn on_stderr(&self, line: &str) {
        self.stderr.lock().unwrap().push(line.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        let input = "\x1b[31mRed\x1b[0m Normal";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Red Normal");
    }

    #[test]
    fn test_captured_output_combined() {
        let output = CapturedOutput {
            stdout: "out".to_string(),
            stderr: "err".to_string(),
            stdout_truncated: false,
            stderr_truncated: false,
            stdout_total_bytes: 3,
            stderr_total_bytes: 3,
        };

        assert_eq!(output.combined(), "out\nerr");
    }

    #[test]
    fn test_captured_output_empty() {
        let output = CapturedOutput::empty();
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert!(!output.is_truncated());
    }

    #[tokio::test]
    async fn test_output_streamer() {
        let streamer = OutputStreamer::new(10);
        let tx = streamer.sender();

        tx.send(OutputLine::Stdout("line1".to_string())).await.unwrap();
        tx.send(OutputLine::Stderr("error1".to_string())).await.unwrap();

        drop(tx);

        let mut streamer = streamer;
        let line1 = streamer.recv().await.unwrap();
        assert!(matches!(line1, OutputLine::Stdout(s) if s == "line1"));

        let line2 = streamer.recv().await.unwrap();
        assert!(matches!(line2, OutputLine::Stderr(s) if s == "error1"));
    }
}
```

---

## Testing Requirements

1. Stdout and stderr are captured separately
2. Large output is truncated at limit
3. ANSI codes are stripped when configured
4. Streaming output works line by line
5. Binary output is handled without panic
6. Output callbacks receive correct data
7. Combined output works correctly

---

## Related Specs

- Depends on: [036-bash-exec-core.md](036-bash-exec-core.md)
- Next: [039-bash-errors.md](039-bash-errors.md)
- Used by: Agent loop for command output display
