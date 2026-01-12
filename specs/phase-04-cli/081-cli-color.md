# Spec 081: ANSI Color Support

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 081
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 079-cli-output
- **Estimated Context**: ~8%

## Objective

Implement ANSI color and styling support for terminal output, with automatic detection of terminal capabilities and user-configurable color preferences.

## Acceptance Criteria

- [ ] `--color` flag with auto/always/never modes
- [ ] Terminal capability detection
- [ ] Semantic color palette (success, error, warning, info)
- [ ] Style support (bold, dim, italic, underline)
- [ ] NO_COLOR environment variable support
- [ ] 256-color and truecolor detection
- [ ] Theme customization support
- [ ] Color stripping for non-TTY output

## Implementation Details

### src/output/color.rs

```rust
//! ANSI color and styling support.

use std::env;
use std::fmt;
use std::io::IsTerminal;

/// Color mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl ColorMode {
    /// Determine if colors should be used
    pub fn should_color(&self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => detect_color_support(),
        }
    }
}

impl From<clap::ColorChoice> for ColorMode {
    fn from(choice: clap::ColorChoice) -> Self {
        match choice {
            clap::ColorChoice::Auto => Self::Auto,
            clap::ColorChoice::Always => Self::Always,
            clap::ColorChoice::Never => Self::Never,
        }
    }
}

/// Detect if the terminal supports colors
pub fn detect_color_support() -> bool {
    // Check NO_COLOR first (https://no-color.org/)
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check CLICOLOR_FORCE
    if env::var("CLICOLOR_FORCE").map(|v| v != "0").unwrap_or(false) {
        return true;
    }

    // Check if stdout is a terminal
    if !std::io::stdout().is_terminal() {
        return false;
    }

    // Check TERM
    match env::var("TERM").as_deref() {
        Ok("dumb") | Ok("") => false,
        _ => true,
    }
}

/// Color depth support level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorDepth {
    None,
    Basic,      // 8 colors
    Extended,   // 256 colors
    TrueColor,  // 16 million colors
}

impl ColorDepth {
    pub fn detect() -> Self {
        if !detect_color_support() {
            return Self::None;
        }

        // Check for truecolor support
        if env::var("COLORTERM")
            .map(|v| v == "truecolor" || v == "24bit")
            .unwrap_or(false)
        {
            return Self::TrueColor;
        }

        // Check TERM for 256 color support
        if let Ok(term) = env::var("TERM") {
            if term.contains("256color") {
                return Self::Extended;
            }
        }

        Self::Basic
    }
}

/// ANSI color codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    // Basic colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    // Bright colors
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    // 256-color palette
    Ansi256(u8),

    // True color RGB
    Rgb(u8, u8, u8),

    // Default terminal color
    Default,
}

impl Color {
    /// Get the ANSI escape code for foreground
    pub fn fg_code(&self) -> String {
        match self {
            Self::Black => "30".to_string(),
            Self::Red => "31".to_string(),
            Self::Green => "32".to_string(),
            Self::Yellow => "33".to_string(),
            Self::Blue => "34".to_string(),
            Self::Magenta => "35".to_string(),
            Self::Cyan => "36".to_string(),
            Self::White => "37".to_string(),
            Self::BrightBlack => "90".to_string(),
            Self::BrightRed => "91".to_string(),
            Self::BrightGreen => "92".to_string(),
            Self::BrightYellow => "93".to_string(),
            Self::BrightBlue => "94".to_string(),
            Self::BrightMagenta => "95".to_string(),
            Self::BrightCyan => "96".to_string(),
            Self::BrightWhite => "97".to_string(),
            Self::Ansi256(n) => format!("38;5;{n}"),
            Self::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
            Self::Default => "39".to_string(),
        }
    }

    /// Get the ANSI escape code for background
    pub fn bg_code(&self) -> String {
        match self {
            Self::Black => "40".to_string(),
            Self::Red => "41".to_string(),
            Self::Green => "42".to_string(),
            Self::Yellow => "43".to_string(),
            Self::Blue => "44".to_string(),
            Self::Magenta => "45".to_string(),
            Self::Cyan => "46".to_string(),
            Self::White => "47".to_string(),
            Self::BrightBlack => "100".to_string(),
            Self::BrightRed => "101".to_string(),
            Self::BrightGreen => "102".to_string(),
            Self::BrightYellow => "103".to_string(),
            Self::BrightBlue => "104".to_string(),
            Self::BrightMagenta => "105".to_string(),
            Self::BrightCyan => "106".to_string(),
            Self::BrightWhite => "107".to_string(),
            Self::Ansi256(n) => format!("48;5;{n}"),
            Self::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
            Self::Default => "49".to_string(),
        }
    }
}

/// Text style modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    Strikethrough,
}

impl Style {
    pub fn code(&self) -> &str {
        match self {
            Self::Bold => "1",
            Self::Dim => "2",
            Self::Italic => "3",
            Self::Underline => "4",
            Self::Blink => "5",
            Self::Reverse => "7",
            Self::Hidden => "8",
            Self::Strikethrough => "9",
        }
    }

    pub fn reset_code(&self) -> &str {
        match self {
            Self::Bold | Self::Dim => "22",
            Self::Italic => "23",
            Self::Underline => "24",
            Self::Blink => "25",
            Self::Reverse => "27",
            Self::Hidden => "28",
            Self::Strikethrough => "29",
        }
    }
}

/// Styled string builder
#[derive(Debug, Clone)]
pub struct Styled {
    content: String,
    fg: Option<Color>,
    bg: Option<Color>,
    styles: Vec<Style>,
    enabled: bool,
}

impl Styled {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            fg: None,
            bg: None,
            styles: Vec::new(),
            enabled: detect_color_support(),
        }
    }

    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.enabled = mode.should_color();
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.styles.push(style);
        self
    }

    pub fn bold(self) -> Self {
        self.style(Style::Bold)
    }

    pub fn dim(self) -> Self {
        self.style(Style::Dim)
    }

    pub fn italic(self) -> Self {
        self.style(Style::Italic)
    }

    pub fn underline(self) -> Self {
        self.style(Style::Underline)
    }

    // Semantic colors
    pub fn success(self) -> Self {
        self.fg(Color::Green)
    }

    pub fn error(self) -> Self {
        self.fg(Color::Red)
    }

    pub fn warning(self) -> Self {
        self.fg(Color::Yellow)
    }

    pub fn info(self) -> Self {
        self.fg(Color::Cyan)
    }

    pub fn hint(self) -> Self {
        self.fg(Color::BrightBlack)
    }
}

impl fmt::Display for Styled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.enabled {
            return write!(f, "{}", self.content);
        }

        let mut codes = Vec::new();

        for style in &self.styles {
            codes.push(style.code().to_string());
        }

        if let Some(fg) = &self.fg {
            codes.push(fg.fg_code());
        }

        if let Some(bg) = &self.bg {
            codes.push(bg.bg_code());
        }

        if codes.is_empty() {
            write!(f, "{}", self.content)
        } else {
            write!(f, "\x1b[{}m{}\x1b[0m", codes.join(";"), self.content)
        }
    }
}

/// Semantic color palette
#[derive(Debug, Clone)]
pub struct Palette {
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub hint: Color,
    pub primary: Color,
    pub secondary: Color,
    pub header: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Cyan,
            hint: Color::BrightBlack,
            primary: Color::BrightWhite,
            secondary: Color::White,
            header: Color::BrightBlue,
        }
    }
}

impl Palette {
    /// Dark theme palette
    pub fn dark() -> Self {
        Self::default()
    }

    /// Light theme palette
    pub fn light() -> Self {
        Self {
            success: Color::Ansi256(28),  // Darker green
            error: Color::Ansi256(160),   // Darker red
            warning: Color::Ansi256(166), // Darker orange
            info: Color::Ansi256(30),     // Darker cyan
            hint: Color::Ansi256(244),    // Light gray
            primary: Color::Black,
            secondary: Color::Ansi256(236),
            header: Color::Ansi256(24),   // Darker blue
        }
    }
}

/// Color context for commands
pub struct ColorContext {
    pub mode: ColorMode,
    pub palette: Palette,
    pub depth: ColorDepth,
}

impl ColorContext {
    pub fn new(mode: ColorMode) -> Self {
        Self {
            mode,
            palette: Palette::default(),
            depth: ColorDepth::detect(),
        }
    }

    pub fn styled(&self, content: impl Into<String>) -> Styled {
        Styled::new(content).with_color_mode(self.mode)
    }

    pub fn success(&self, content: impl Into<String>) -> Styled {
        self.styled(content).fg(self.palette.success)
    }

    pub fn error(&self, content: impl Into<String>) -> Styled {
        self.styled(content).fg(self.palette.error)
    }

    pub fn warning(&self, content: impl Into<String>) -> Styled {
        self.styled(content).fg(self.palette.warning)
    }

    pub fn info(&self, content: impl Into<String>) -> Styled {
        self.styled(content).fg(self.palette.info)
    }

    pub fn header(&self, content: impl Into<String>) -> Styled {
        self.styled(content).fg(self.palette.header).bold()
    }
}

/// Convenience functions
pub fn styled(content: impl Into<String>) -> Styled {
    Styled::new(content)
}

pub fn success(content: impl Into<String>) -> Styled {
    styled(content).success()
}

pub fn error(content: impl Into<String>) -> Styled {
    styled(content).error()
}

pub fn warning(content: impl Into<String>) -> Styled {
    styled(content).warning()
}

pub fn info(content: impl Into<String>) -> Styled {
    styled(content).info()
}

pub fn bold(content: impl Into<String>) -> Styled {
    styled(content).bold()
}

/// Strip ANSI codes from a string
pub fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}
```

### Icon Support

```rust
//! Unicode icons for CLI output.

/// Status icons
pub struct Icons;

impl Icons {
    pub const CHECK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";
    pub const WARNING: &'static str = "⚠";
    pub const INFO: &'static str = "ℹ";
    pub const ARROW: &'static str = "→";
    pub const BULLET: &'static str = "•";
    pub const SPINNER: [&'static str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    /// Fallback ASCII versions
    pub const CHECK_ASCII: &'static str = "[ok]";
    pub const CROSS_ASCII: &'static str = "[err]";
    pub const WARNING_ASCII: &'static str = "[warn]";
    pub const INFO_ASCII: &'static str = "[info]";
    pub const ARROW_ASCII: &'static str = "->";
    pub const BULLET_ASCII: &'static str = "*";
}

/// Icon context that handles unicode support detection
pub struct IconContext {
    unicode: bool,
}

impl IconContext {
    pub fn new() -> Self {
        Self {
            unicode: detect_unicode_support(),
        }
    }

    pub fn check(&self) -> &'static str {
        if self.unicode { Icons::CHECK } else { Icons::CHECK_ASCII }
    }

    pub fn cross(&self) -> &'static str {
        if self.unicode { Icons::CROSS } else { Icons::CROSS_ASCII }
    }

    pub fn warning(&self) -> &'static str {
        if self.unicode { Icons::WARNING } else { Icons::WARNING_ASCII }
    }

    pub fn info(&self) -> &'static str {
        if self.unicode { Icons::INFO } else { Icons::INFO_ASCII }
    }
}

impl Default for IconContext {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_unicode_support() -> bool {
    env::var("TERM")
        .map(|t| !t.contains("linux"))
        .unwrap_or(true)
        && env::var("LANG")
            .map(|l| l.to_uppercase().contains("UTF"))
            .unwrap_or(true)
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_no_color() {
        let styled = Styled::new("test").with_color_mode(ColorMode::Never);
        assert_eq!(styled.to_string(), "test");
    }

    #[test]
    fn test_styled_with_color() {
        let styled = Styled::new("test")
            .with_color_mode(ColorMode::Always)
            .fg(Color::Red);
        assert!(styled.to_string().contains("\x1b[31m"));
        assert!(styled.to_string().contains("\x1b[0m"));
    }

    #[test]
    fn test_styled_bold() {
        let styled = Styled::new("test")
            .with_color_mode(ColorMode::Always)
            .bold();
        assert!(styled.to_string().contains("\x1b[1m"));
    }

    #[test]
    fn test_color_codes() {
        assert_eq!(Color::Red.fg_code(), "31");
        assert_eq!(Color::Ansi256(42).fg_code(), "38;5;42");
        assert_eq!(Color::Rgb(255, 128, 0).fg_code(), "38;2;255;128;0");
    }

    #[test]
    fn test_strip_ansi() {
        let colored = "\x1b[31mred\x1b[0m text";
        assert_eq!(strip_ansi(colored), "red text");
    }

    #[test]
    fn test_palette_default() {
        let palette = Palette::default();
        assert_eq!(palette.success, Color::Green);
        assert_eq!(palette.error, Color::Red);
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **079-cli-output.md**: Output formatting
- **082-cli-progress.md**: Progress indicators
- **091-cli-errors.md**: Error output styling
