//! Progress indicators for CLI operations.

use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::output::color::{ColorMode, Styled, Color};

/// Spinner animation frames
pub const SPINNER_DOTS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
pub const SPINNER_LINE: &[&str] = &["-", "\\", "|", "/"];
pub const SPINNER_ARROWS: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];
pub const SPINNER_BRAILLE: &[&str] = &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];

/// Progress bar style
#[derive(Debug, Clone)]
pub struct ProgressStyle {
    pub filled: char,
    pub current: char,
    pub empty: char,
    pub brackets: (char, char),
    pub width: usize,
    pub show_percentage: bool,
    pub show_count: bool,
    pub show_eta: bool,
    pub show_speed: bool,
    pub color: Option<Color>,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            filled: '█',
            current: '█',
            empty: '░',
            brackets: ('[', ']'),
            width: 40,
            show_percentage: true,
            show_count: true,
            show_eta: true,
            show_speed: false,
            color: Some(Color::Cyan),
        }
    }
}

impl ProgressStyle {
    pub fn ascii() -> Self {
        Self {
            filled: '=',
            current: '>',
            empty: '-',
            brackets: ('[', ']'),
            ..Default::default()
        }
    }

    pub fn blocks() -> Self {
        Self {
            filled: '█',
            current: '▓',
            empty: '░',
            brackets: ('│', '│'),
            ..Default::default()
        }
    }

    pub fn minimal() -> Self {
        Self {
            filled: '●',
            current: '●',
            empty: '○',
            brackets: (' ', ' '),
            width: 20,
            show_eta: false,
            show_count: false,
            ..Default::default()
        }
    }
}

/// Spinner for indeterminate progress
pub struct Spinner {
    message: String,
    frames: &'static [&'static str],
    interval: Duration,
    running: Arc<AtomicBool>,
    color_mode: ColorMode,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frames: SPINNER_DOTS,
            interval: Duration::from_millis(80),
            running: Arc::new(AtomicBool::new(false)),
            color_mode: ColorMode::Auto,
        }
    }

    pub fn frames(mut self, frames: &'static [&'static str]) -> Self {
        self.frames = frames;
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Start the spinner (returns handle to stop it)
    pub fn start(self) -> SpinnerHandle {
        let is_tty = io::stdout().is_terminal();
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let message = self.message.clone();
        let frames = self.frames;
        let interval = self.interval;
        let color_mode = self.color_mode;

        let handle = std::thread::spawn(move || {
            let mut frame_idx = 0;

            while running.load(Ordering::SeqCst) {
                let frame = frames[frame_idx % frames.len()];

                if is_tty {
                    let line = if color_mode.should_color() {
                        format!("\r\x1b[36m{frame}\x1b[0m {message}")
                    } else {
                        format!("\r{frame} {message}")
                    };
                    print!("{line}");
                    let _ = io::stdout().flush();
                }

                std::thread::sleep(interval);
                frame_idx += 1;
            }

            // Clear spinner line
            if is_tty {
                print!("\r\x1b[K");
                let _ = io::stdout().flush();
            }
        });

        SpinnerHandle {
            running: self.running,
            thread: Some(handle),
        }
    }
}

/// Handle to control a running spinner
pub struct SpinnerHandle {
    running: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl SpinnerHandle {
    /// Stop the spinner with a success message
    pub fn finish(self, message: &str) {
        self.stop_with_symbol("✓", Color::Green, message);
    }

    /// Stop the spinner with an error message
    pub fn fail(self, message: &str) {
        self.stop_with_symbol("✗", Color::Red, message);
    }

    /// Stop the spinner with a warning message
    pub fn warn(self, message: &str) {
        self.stop_with_symbol("⚠", Color::Yellow, message);
    }

    /// Stop the spinner with a custom symbol
    pub fn stop_with_symbol(mut self, symbol: &str, color: Color, message: &str) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }

        if io::stdout().is_terminal() {
            let styled = Styled::new(symbol).fg(color);
            println!("{styled} {message}");
        }
    }

    /// Stop the spinner silently
    pub fn stop(mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for SpinnerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Progress bar for determinate progress
pub struct ProgressBar {
    total: u64,
    current: Arc<AtomicU64>,
    message: String,
    style: ProgressStyle,
    started: Instant,
    color_mode: ColorMode,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: Arc::new(AtomicU64::new(0)),
            message: String::new(),
            style: ProgressStyle::default(),
            started: Instant::now(),
            color_mode: ColorMode::Auto,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Increment progress by 1
    pub fn inc(&self) {
        self.inc_by(1);
    }

    /// Increment progress by n
    pub fn inc_by(&self, n: u64) {
        self.current.fetch_add(n, Ordering::SeqCst);
        self.render();
    }

    /// Set absolute progress
    pub fn set(&self, value: u64) {
        self.current.store(value, Ordering::SeqCst);
        self.render();
    }

    /// Update the message
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.render();
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        self.current.store(self.total, Ordering::SeqCst);
        self.render();
        println!();
    }

    /// Finish with a message
    pub fn finish_with_message(&self, message: &str) {
        self.current.store(self.total, Ordering::SeqCst);
        if io::stdout().is_terminal() {
            print!("\r\x1b[K");
            let symbol = Styled::new("✓").fg(Color::Green);
            println!("{symbol} {message}");
        }
    }

    fn render(&self) {
        if !io::stdout().is_terminal() {
            return;
        }

        let current = self.current.load(Ordering::SeqCst);
        let percent = if self.total > 0 {
            (current as f64 / self.total as f64 * 100.0) as u8
        } else {
            0
        };

        let filled = (self.style.width as f64 * current as f64 / self.total as f64) as usize;
        let empty = self.style.width.saturating_sub(filled);

        let bar = format!(
            "{}{}{}{}{}",
            self.style.brackets.0,
            self.style.filled.to_string().repeat(filled.saturating_sub(1)),
            if filled > 0 { self.style.current } else { self.style.empty },
            self.style.empty.to_string().repeat(empty),
            self.style.brackets.1,
        );

        let mut parts = vec![];

        if self.style.show_percentage {
            parts.push(format!("{percent:3}%"));
        }

        if self.style.show_count {
            parts.push(format!("{current}/{}", self.total));
        }

        if self.style.show_eta && current > 0 {
            let elapsed = self.started.elapsed().as_secs_f64();
            let rate = current as f64 / elapsed;
            let remaining = (self.total - current) as f64 / rate;
            parts.push(format!("ETA: {}", format_duration(remaining)));
        }

        if self.style.show_speed && current > 0 {
            let elapsed = self.started.elapsed().as_secs_f64();
            let rate = current as f64 / elapsed;
            parts.push(format!("{:.1}/s", rate));
        }

        let info = parts.join(" ");

        let line = if self.message.is_empty() {
            format!("\r{bar} {info}")
        } else {
            format!("\r{bar} {info} - {}", self.message)
        };

        if self.color_mode.should_color() {
            if let Some(color) = self.style.color {
                let styled_bar = Styled::new(&bar).fg(color);
                print!("\r{styled_bar} {info}");
            } else {
                print!("{line}");
            }
        } else {
            print!("{line}");
        }

        let _ = io::stdout().flush();
    }
}

fn format_duration(secs: f64) -> String {
    if secs < 60.0 {
        format!("{:.0}s", secs)
    } else if secs < 3600.0 {
        format!("{}m {}s", (secs / 60.0) as u32, (secs % 60.0) as u32)
    } else {
        format!(
            "{}h {}m",
            (secs / 3600.0) as u32,
            ((secs % 3600.0) / 60.0) as u32
        )
    }
}

/// Multi-progress display for parallel operations
pub struct MultiProgress {
    bars: Vec<(String, Arc<AtomicU64>, u64)>,
    running: Arc<AtomicBool>,
}

impl MultiProgress {
    pub fn new() -> Self {
        Self {
            bars: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Add a progress bar and return its index
    pub fn add(&mut self, name: impl Into<String>, total: u64) -> usize {
        let idx = self.bars.len();
        self.bars.push((name.into(), Arc::new(AtomicU64::new(0)), total));
        idx
    }

    /// Increment a specific bar
    pub fn inc(&self, idx: usize) {
        if let Some((_, current, _)) = self.bars.get(idx) {
            current.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Set progress for a specific bar
    pub fn set(&self, idx: usize, value: u64) {
        if let Some((_, current, _)) = self.bars.get(idx) {
            current.store(value, Ordering::SeqCst);
        }
    }

    /// Start rendering the multi-progress display
    pub fn start(&self) -> MultiProgressHandle {
        let bars = self.bars.clone();
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let handle = std::thread::spawn(move || {
            let is_tty = io::stdout().is_terminal();

            while running.load(Ordering::SeqCst) {
                if is_tty {
                    // Move cursor up to redraw
                    print!("\x1b[{}A", bars.len());

                    for (name, current, total) in &bars {
                        let cur = current.load(Ordering::SeqCst);
                        let percent = if *total > 0 {
                            (cur as f64 / *total as f64 * 100.0) as u8
                        } else {
                            0
                        };

                        let width = 30;
                        let filled = (width as f64 * cur as f64 / *total as f64) as usize;
                        let bar: String = "█".repeat(filled) + &"░".repeat(width - filled);

                        println!("\x1b[K{name:20} [{bar}] {percent:3}% ({cur}/{total})");
                    }

                    let _ = io::stdout().flush();
                }

                std::thread::sleep(Duration::from_millis(100));
            }
        });

        // Print initial lines
        if io::stdout().is_terminal() {
            for _ in &self.bars {
                println!();
            }
        }

        MultiProgressHandle {
            running: self.running.clone(),
            thread: Some(handle),
        }
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MultiProgressHandle {
    running: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MultiProgressHandle {
    pub fn finish(mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for MultiProgressHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Async-compatible progress updates
pub struct AsyncProgress {
    tx: mpsc::Sender<ProgressUpdate>,
}

pub enum ProgressUpdate {
    Inc(u64),
    Set(u64),
    Message(String),
    Finish,
}

impl AsyncProgress {
    pub async fn inc(&self) {
        let _ = self.tx.send(ProgressUpdate::Inc(1)).await;
    }

    pub async fn inc_by(&self, n: u64) {
        let _ = self.tx.send(ProgressUpdate::Inc(n)).await;
    }

    pub async fn set(&self, value: u64) {
        let _ = self.tx.send(ProgressUpdate::Set(value)).await;
    }

    pub async fn message(&self, msg: impl Into<String>) {
        let _ = self.tx.send(ProgressUpdate::Message(msg.into())).await;
    }

    pub async fn finish(&self) {
        let _ = self.tx.send(ProgressUpdate::Finish).await;
    }
}

/// Create an async progress bar
pub fn async_progress(total: u64) -> (AsyncProgress, impl std::future::Future<Output = ()>) {
    let (tx, mut rx) = mpsc::channel(100);

    let progress = AsyncProgress { tx };

    let runner = async move {
        let mut bar = ProgressBar::new(total);

        while let Some(update) = rx.recv().await {
            match update {
                ProgressUpdate::Inc(n) => bar.inc_by(n),
                ProgressUpdate::Set(v) => bar.set(v),
                ProgressUpdate::Message(m) => bar.set_message(m),
                ProgressUpdate::Finish => {
                    bar.finish();
                    break;
                }
            }
        }
    };

    (progress, runner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation() {
        let bar = ProgressBar::new(100);
        assert_eq!(bar.total, 100);
        assert_eq!(bar.current.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_progress_style_ascii() {
        let style = ProgressStyle::ascii();
        assert_eq!(style.filled, '=');
        assert_eq!(style.current, '>');
        assert_eq!(style.empty, '-');
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30.0), "30s");
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(3700.0), "1h 1m");
    }

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new("Loading...");
        assert_eq!(spinner.message, "Loading...");
        assert_eq!(spinner.frames.len(), SPINNER_DOTS.len());
    }

    #[test]
    fn test_multi_progress() {
        let mut mp = MultiProgress::new();
        let idx1 = mp.add("Task 1", 100);
        let idx2 = mp.add("Task 2", 50);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);

        mp.inc(idx1);
        mp.set(idx2, 25);

        assert_eq!(mp.bars[0].1.load(Ordering::SeqCst), 1);
        assert_eq!(mp.bars[1].1.load(Ordering::SeqCst), 25);
    }

    #[test]
    fn test_progress_styles() {
        let default = ProgressStyle::default();
        assert_eq!(default.filled, '█');
        assert_eq!(default.width, 40);
        assert!(default.show_percentage);

        let minimal = ProgressStyle::minimal();
        assert_eq!(minimal.width, 20);
        assert!(!minimal.show_eta);
        assert!(!minimal.show_count);

        let blocks = ProgressStyle::blocks();
        assert_eq!(blocks.brackets, ('│', '│'));
        assert_eq!(blocks.current, '▓');
    }

    #[test]
    fn test_spinner_frames() {
        let spinner = Spinner::new("Test")
            .frames(SPINNER_LINE)
            .interval(Duration::from_millis(50));

        assert_eq!(spinner.frames, SPINNER_LINE);
        assert_eq!(spinner.interval, Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_async_progress() {
        let (progress, runner) = async_progress(100);
        
        // Spawn the runner
        let handle = tokio::spawn(runner);
        
        // Send some updates
        progress.inc().await;
        progress.inc_by(5).await;
        progress.set(50).await;
        progress.message("Testing...".to_string()).await;
        progress.finish().await;
        
        // Wait for completion
        handle.await.unwrap();
    }

    #[test]
    fn test_progress_update_enum() {
        let update = ProgressUpdate::Inc(5);
        matches!(update, ProgressUpdate::Inc(5));
        
        let update = ProgressUpdate::Set(100);
        matches!(update, ProgressUpdate::Set(100));
        
        let update = ProgressUpdate::Message("test".to_string());
        matches!(update, ProgressUpdate::Message(_));
        
        let update = ProgressUpdate::Finish;
        matches!(update, ProgressUpdate::Finish);
    }
}