//! Output capture for bash commands.

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::mpsc;

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
    /// Total bytes received before truncation.
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
    /// Standard output line.
    Stdout(String),
    /// Standard error line.
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
    /// Called when stdout line is received.
    fn on_stdout(&self, line: &str);
    /// Called when stderr line is received.  
    fn on_stderr(&self, line: &str);
}

/// A simple callback that collects output.
pub struct CollectingCallback {
    stdout: std::sync::Mutex<Vec<String>>,
    stderr: std::sync::Mutex<Vec<String>>,
}

impl CollectingCallback {
    /// Create a new collecting callback.
    pub fn new() -> Self {
        Self {
            stdout: std::sync::Mutex::new(Vec::new()),
            stderr: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get collected stdout lines.
    pub fn stdout(&self) -> Vec<String> {
        self.stdout.lock().unwrap().clone()
    }

    /// Get collected stderr lines.
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
    fn test_strip_ansi_complex() {
        let input = "\x1b[1;32mBold Green\x1b[22;39m \x1b[4mUnderlined\x1b[24m";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Bold Green Underlined");
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
    fn test_captured_output_combined_empty_stderr() {
        let output = CapturedOutput {
            stdout: "only stdout".to_string(),
            stderr: "".to_string(),
            stdout_truncated: false,
            stderr_truncated: false,
            stdout_total_bytes: 11,
            stderr_total_bytes: 0,
        };

        assert_eq!(output.combined(), "only stdout");
    }

    #[test]
    fn test_captured_output_combined_empty_stdout() {
        let output = CapturedOutput {
            stdout: "".to_string(),
            stderr: "only stderr".to_string(),
            stdout_truncated: false,
            stderr_truncated: false,
            stdout_total_bytes: 0,
            stderr_total_bytes: 11,
        };

        assert_eq!(output.combined(), "only stderr");
    }

    #[test]
    fn test_captured_output_empty() {
        let output = CapturedOutput::empty();
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert!(!output.is_truncated());
        assert_eq!(output.stdout_total_bytes, 0);
        assert_eq!(output.stderr_total_bytes, 0);
    }

    #[test]
    fn test_captured_output_truncated() {
        let output = CapturedOutput {
            stdout: "truncated".to_string(),
            stderr: "".to_string(),
            stdout_truncated: true,
            stderr_truncated: false,
            stdout_total_bytes: 1000,
            stderr_total_bytes: 0,
        };

        assert!(output.is_truncated());
        assert_eq!(output.stdout_total_bytes, 1000);
    }

    #[test]
    fn test_output_config_default() {
        let config = OutputConfig::default();
        assert_eq!(config.max_stdout, DEFAULT_MAX_OUTPUT);
        assert_eq!(config.max_stderr, DEFAULT_MAX_OUTPUT);
        assert!(config.strip_ansi);
        assert!(config.trim_output);
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

        // Should return None when sender is dropped
        let end = streamer.recv().await;
        assert!(end.is_none());
    }

    #[test]
    fn test_collecting_callback() {
        let callback = CollectingCallback::new();
        
        callback.on_stdout("stdout line 1");
        callback.on_stdout("stdout line 2");
        callback.on_stderr("stderr line 1");
        
        let stdout_lines = callback.stdout();
        let stderr_lines = callback.stderr();
        
        assert_eq!(stdout_lines.len(), 2);
        assert_eq!(stdout_lines[0], "stdout line 1");
        assert_eq!(stdout_lines[1], "stdout line 2");
        
        assert_eq!(stderr_lines.len(), 1);
        assert_eq!(stderr_lines[0], "stderr line 1");
    }

    #[tokio::test]
    async fn test_streaming_output() {
        // Test OutputStreamer with a mock process
        let mut streamer = OutputStreamer::new(10);
        let tx = streamer.sender();

        // Simulate lines coming from a process
        let sender_task = tokio::spawn(async move {
            tx.send(OutputLine::Stdout("Line 1".to_string())).await.unwrap();
            tx.send(OutputLine::Stderr("Error 1".to_string())).await.unwrap();
            tx.send(OutputLine::Stdout("Line 2".to_string())).await.unwrap();
            // Drop sender to signal end
        });

        // Collect streamed output
        let mut received = Vec::new();
        while let Some(line) = streamer.recv().await {
            received.push(line);
        }

        sender_task.await.unwrap();

        assert_eq!(received.len(), 3);
        assert!(matches!(received[0], OutputLine::Stdout(ref s) if s == "Line 1"));
        assert!(matches!(received[1], OutputLine::Stderr(ref s) if s == "Error 1"));
        assert!(matches!(received[2], OutputLine::Stdout(ref s) if s == "Line 2"));
    }
}