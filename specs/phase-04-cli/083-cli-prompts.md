# Spec 083: Interactive Prompts

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 083
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 081-cli-color
- **Estimated Context**: ~10%

## Objective

Implement interactive prompt utilities for user input, confirmation dialogs, selection menus, and form-style data collection in the CLI.

## Acceptance Criteria

- [ ] Confirmation prompts (yes/no)
- [ ] Text input with validation
- [ ] Password input (masked)
- [ ] Single selection from list
- [ ] Multi-selection from list
- [ ] Number input with range validation
- [ ] Path input with completion
- [ ] Non-interactive fallback mode
- [ ] Default values support

## Implementation Details

### src/prompts/mod.rs

```rust
//! Interactive prompts for CLI user input.

mod confirm;
mod input;
mod multiselect;
mod password;
mod select;

pub use confirm::Confirm;
pub use input::Input;
pub use multiselect::MultiSelect;
pub use password::Password;
pub use select::Select;

use std::io::{self, IsTerminal, Write};

use crate::output::color::{Color, ColorMode, Styled};

/// Check if we're in interactive mode
pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// Prompt theme
#[derive(Debug, Clone)]
pub struct Theme {
    pub prompt_prefix: String,
    pub prompt_suffix: String,
    pub selected_prefix: String,
    pub unselected_prefix: String,
    pub active_color: Color,
    pub error_color: Color,
    pub hint_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            prompt_prefix: "?".to_string(),
            prompt_suffix: "›".to_string(),
            selected_prefix: "◉".to_string(),
            unselected_prefix: "○".to_string(),
            active_color: Color::Cyan,
            error_color: Color::Red,
            hint_color: Color::BrightBlack,
        }
    }
}

impl Theme {
    pub fn ascii() -> Self {
        Self {
            prompt_prefix: "?".to_string(),
            prompt_suffix: ">".to_string(),
            selected_prefix: "[x]".to_string(),
            unselected_prefix: "[ ]".to_string(),
            ..Default::default()
        }
    }
}

/// Result type for prompts
pub type PromptResult<T> = Result<T, PromptError>;

/// Errors that can occur during prompts
#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("Input cancelled by user")]
    Cancelled,

    #[error("Not running in interactive mode")]
    NotInteractive,

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
```

### src/prompts/confirm.rs

```rust
//! Confirmation prompt (yes/no).

use std::io::{self, BufRead, Write};

use super::{is_interactive, PromptError, PromptResult, Theme};
use crate::output::color::{ColorMode, Styled};

/// Confirmation prompt builder
pub struct Confirm {
    message: String,
    default: Option<bool>,
    theme: Theme,
    color_mode: ColorMode,
}

impl Confirm {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            theme: Theme::default(),
            color_mode: ColorMode::Auto,
        }
    }

    /// Set default value (shown when user presses Enter)
    pub fn default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set color mode
    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Run the prompt
    pub fn prompt(self) -> PromptResult<bool> {
        if !is_interactive() {
            return self.default.ok_or(PromptError::NotInteractive);
        }

        let hint = match self.default {
            Some(true) => "(Y/n)",
            Some(false) => "(y/N)",
            None => "(y/n)",
        };

        let prefix = Styled::new(&self.theme.prompt_prefix)
            .with_color_mode(self.color_mode)
            .fg(self.theme.active_color);

        print!("{prefix} {} {hint} ", self.message);
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "" => self.default.ok_or(PromptError::ValidationFailed(
                "Please enter y or n".to_string(),
            )),
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(PromptError::ValidationFailed(
                "Please enter y or n".to_string(),
            )),
        }
    }

    /// Run prompt with retry on invalid input
    pub fn prompt_until_valid(self) -> PromptResult<bool> {
        loop {
            match self.clone().prompt() {
                Ok(result) => return Ok(result),
                Err(PromptError::ValidationFailed(msg)) => {
                    eprintln!(
                        "{} {}",
                        Styled::new("!").fg(self.theme.error_color),
                        msg
                    );
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl Clone for Confirm {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            default: self.default,
            theme: self.theme.clone(),
            color_mode: self.color_mode,
        }
    }
}

/// Convenience function for simple confirmation
pub fn confirm(message: &str) -> PromptResult<bool> {
    Confirm::new(message).default(false).prompt()
}

/// Convenience function for confirmation with default true
pub fn confirm_default_yes(message: &str) -> PromptResult<bool> {
    Confirm::new(message).default(true).prompt()
}
```

### src/prompts/input.rs

```rust
//! Text input prompt.

use std::io::{self, BufRead, Write};

use super::{is_interactive, PromptError, PromptResult, Theme};
use crate::output::color::{ColorMode, Styled};

/// Validation function type
pub type Validator<T> = Box<dyn Fn(&str) -> Result<T, String>>;

/// Text input prompt builder
pub struct Input<T> {
    message: String,
    default: Option<String>,
    placeholder: Option<String>,
    validator: Option<Validator<T>>,
    theme: Theme,
    color_mode: ColorMode,
    allow_empty: bool,
}

impl<T> Input<T> {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            placeholder: None,
            validator: None,
            theme: Theme::default(),
            color_mode: ColorMode::Auto,
            allow_empty: false,
        }
    }

    /// Set default value
    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set validator function
    pub fn validate<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Result<T, String> + 'static,
    {
        self.validator = Some(Box::new(f));
        self
    }

    /// Allow empty input
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set color mode
    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }
}

impl Input<String> {
    /// Run the prompt (string version)
    pub fn prompt(self) -> PromptResult<String> {
        if !is_interactive() {
            return self.default.ok_or(PromptError::NotInteractive);
        }

        let prefix = Styled::new(&self.theme.prompt_prefix)
            .with_color_mode(self.color_mode)
            .fg(self.theme.active_color);

        let hint = match (&self.default, &self.placeholder) {
            (Some(d), _) => format!("(default: {d})"),
            (_, Some(p)) => format!("({p})"),
            _ => String::new(),
        };

        let hint_styled = Styled::new(&hint)
            .with_color_mode(self.color_mode)
            .fg(self.theme.hint_color);

        print!("{prefix} {} {hint_styled} ", self.message);
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            if let Some(default) = self.default {
                return Ok(default);
            }
            if !self.allow_empty {
                return Err(PromptError::ValidationFailed(
                    "Input cannot be empty".to_string(),
                ));
            }
        }

        if let Some(validator) = self.validator {
            validator(input).map_err(PromptError::ValidationFailed)
        } else {
            Ok(input.to_string())
        }
    }
}

impl<T> Input<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    /// Run the prompt with parsing
    pub fn prompt_parsed(self) -> PromptResult<T> {
        if !is_interactive() {
            if let Some(default) = &self.default {
                return default
                    .parse()
                    .map_err(|e: <T as std::str::FromStr>::Err| {
                        PromptError::ValidationFailed(e.to_string())
                    });
            }
            return Err(PromptError::NotInteractive);
        }

        let prefix = Styled::new(&self.theme.prompt_prefix)
            .with_color_mode(self.color_mode)
            .fg(self.theme.active_color);

        print!("{prefix} {} ", self.message);
        if let Some(default) = &self.default {
            let hint = Styled::new(format!("(default: {default})"))
                .with_color_mode(self.color_mode)
                .fg(self.theme.hint_color);
            print!("{hint} ");
        }
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            if let Some(default) = self.default {
                return default
                    .parse()
                    .map_err(|e: <T as std::str::FromStr>::Err| {
                        PromptError::ValidationFailed(e.to_string())
                    });
            }
        }

        input
            .parse()
            .map_err(|e: <T as std::str::FromStr>::Err| {
                PromptError::ValidationFailed(e.to_string())
            })
    }
}

/// Convenience function for simple text input
pub fn input(message: &str) -> PromptResult<String> {
    Input::new(message).prompt()
}

/// Convenience function for text input with default
pub fn input_with_default(message: &str, default: &str) -> PromptResult<String> {
    Input::new(message).default(default).prompt()
}
```

### src/prompts/password.rs

```rust
//! Password input prompt (masked).

use std::io::{self, Write};

use super::{is_interactive, PromptError, PromptResult, Theme};
use crate::output::color::{ColorMode, Styled};

/// Password input prompt
pub struct Password {
    message: String,
    confirm: bool,
    confirm_message: String,
    theme: Theme,
    color_mode: ColorMode,
}

impl Password {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            confirm: false,
            confirm_message: "Confirm password".to_string(),
            theme: Theme::default(),
            color_mode: ColorMode::Auto,
        }
    }

    /// Require password confirmation
    pub fn with_confirmation(mut self) -> Self {
        self.confirm = true;
        self
    }

    /// Set confirmation message
    pub fn confirm_message(mut self, message: impl Into<String>) -> Self {
        self.confirm_message = message.into();
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set color mode
    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Run the prompt
    pub fn prompt(self) -> PromptResult<String> {
        if !is_interactive() {
            return Err(PromptError::NotInteractive);
        }

        let password = self.read_password(&self.message)?;

        if self.confirm {
            let confirmed = self.read_password(&self.confirm_message)?;
            if password != confirmed {
                return Err(PromptError::ValidationFailed(
                    "Passwords do not match".to_string(),
                ));
            }
        }

        Ok(password)
    }

    fn read_password(&self, message: &str) -> PromptResult<String> {
        let prefix = Styled::new(&self.theme.prompt_prefix)
            .with_color_mode(self.color_mode)
            .fg(self.theme.active_color);

        print!("{prefix} {message}: ");
        io::stdout().flush()?;

        // Use rpassword for cross-platform password reading
        let password = rpassword::read_password()?;

        Ok(password)
    }
}

/// Convenience function for password input
pub fn password(message: &str) -> PromptResult<String> {
    Password::new(message).prompt()
}

/// Convenience function for password with confirmation
pub fn password_with_confirmation(message: &str) -> PromptResult<String> {
    Password::new(message).with_confirmation().prompt()
}
```

### src/prompts/select.rs

```rust
//! Single selection prompt.

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};

use super::{is_interactive, PromptError, PromptResult, Theme};
use crate::output::color::{Color, ColorMode, Styled};

/// Selection option
#[derive(Debug, Clone)]
pub struct SelectOption<T> {
    pub value: T,
    pub label: String,
    pub hint: Option<String>,
}

impl<T> SelectOption<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Single selection prompt
pub struct Select<T> {
    message: String,
    options: Vec<SelectOption<T>>,
    default: usize,
    theme: Theme,
    color_mode: ColorMode,
    page_size: usize,
}

impl<T> Select<T> {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            options: Vec::new(),
            default: 0,
            theme: Theme::default(),
            color_mode: ColorMode::Auto,
            page_size: 10,
        }
    }

    /// Add an option
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// Add options from iterator
    pub fn options(mut self, options: impl IntoIterator<Item = SelectOption<T>>) -> Self {
        self.options.extend(options);
        self
    }

    /// Add simple string options
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self
    where
        T: ToString + Clone,
    {
        for item in items {
            let label = item.to_string();
            self.options.push(SelectOption::new(item, label));
        }
        self
    }

    /// Set default selection index
    pub fn default(mut self, index: usize) -> Self {
        self.default = index;
        self
    }

    /// Set page size
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set color mode
    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Run the prompt
    pub fn prompt(self) -> PromptResult<T>
    where
        T: Clone,
    {
        if self.options.is_empty() {
            return Err(PromptError::ValidationFailed(
                "No options provided".to_string(),
            ));
        }

        if !is_interactive() {
            // Return default option
            return Ok(self.options[self.default.min(self.options.len() - 1)]
                .value
                .clone());
        }

        terminal::enable_raw_mode()?;
        let result = self.run_interactive();
        terminal::disable_raw_mode()?;

        result
    }

    fn run_interactive(self) -> PromptResult<T>
    where
        T: Clone,
    {
        let mut stdout = io::stdout();
        let mut selected = self.default.min(self.options.len() - 1);
        let mut scroll_offset = 0;

        // Print prompt message
        let prefix = Styled::new(&self.theme.prompt_prefix)
            .with_color_mode(self.color_mode)
            .fg(self.theme.active_color);
        println!("{prefix} {}", self.message);

        loop {
            // Calculate visible range
            let visible_end = (scroll_offset + self.page_size).min(self.options.len());

            // Render options
            for (i, option) in self.options[scroll_offset..visible_end].iter().enumerate() {
                let idx = scroll_offset + i;
                let is_selected = idx == selected;

                let prefix = if is_selected {
                    Styled::new("❯")
                        .with_color_mode(self.color_mode)
                        .fg(self.theme.active_color)
                } else {
                    Styled::new(" ").with_color_mode(self.color_mode)
                };

                let label = if is_selected {
                    Styled::new(&option.label)
                        .with_color_mode(self.color_mode)
                        .fg(self.theme.active_color)
                } else {
                    Styled::new(&option.label).with_color_mode(self.color_mode)
                };

                if let Some(hint) = &option.hint {
                    let hint_styled = Styled::new(hint)
                        .with_color_mode(self.color_mode)
                        .fg(self.theme.hint_color);
                    println!("{prefix} {label} {hint_styled}");
                } else {
                    println!("{prefix} {label}");
                }
            }

            // Handle input
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                            if selected < scroll_offset {
                                scroll_offset = selected;
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < self.options.len() - 1 {
                            selected += 1;
                            if selected >= scroll_offset + self.page_size {
                                scroll_offset = selected - self.page_size + 1;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        // Clear selection display
                        execute!(
                            stdout,
                            cursor::MoveUp(visible_end as u16 - scroll_offset as u16),
                            terminal::Clear(ClearType::FromCursorDown)
                        )?;

                        // Show selected value
                        let selected_styled = Styled::new(&self.options[selected].label)
                            .with_color_mode(self.color_mode)
                            .fg(self.theme.active_color);
                        println!("{selected_styled}");

                        return Ok(self.options[selected].value.clone());
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        execute!(
                            stdout,
                            cursor::MoveUp(visible_end as u16 - scroll_offset as u16),
                            terminal::Clear(ClearType::FromCursorDown)
                        )?;
                        return Err(PromptError::Cancelled);
                    }
                    _ => {}
                }
            }

            // Move cursor up to redraw
            execute!(
                stdout,
                cursor::MoveUp(visible_end as u16 - scroll_offset as u16),
                terminal::Clear(ClearType::FromCursorDown)
            )?;
        }
    }
}

/// Convenience function for simple selection
pub fn select<T: Clone + ToString>(message: &str, items: Vec<T>) -> PromptResult<T> {
    Select::new(message).items(items).prompt()
}
```

### src/prompts/multiselect.rs

```rust
//! Multi-selection prompt.

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType},
};

use super::{is_interactive, PromptError, PromptResult, SelectOption, Theme};
use crate::output::color::{ColorMode, Styled};

/// Multi-selection prompt
pub struct MultiSelect<T> {
    message: String,
    options: Vec<SelectOption<T>>,
    defaults: Vec<bool>,
    min_selections: Option<usize>,
    max_selections: Option<usize>,
    theme: Theme,
    color_mode: ColorMode,
    page_size: usize,
}

impl<T> MultiSelect<T> {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            options: Vec::new(),
            defaults: Vec::new(),
            min_selections: None,
            max_selections: None,
            theme: Theme::default(),
            color_mode: ColorMode::Auto,
            page_size: 10,
        }
    }

    /// Add an option
    pub fn option(mut self, option: SelectOption<T>, selected: bool) -> Self {
        self.defaults.push(selected);
        self.options.push(option);
        self
    }

    /// Add options from iterator
    pub fn options(
        mut self,
        options: impl IntoIterator<Item = (SelectOption<T>, bool)>,
    ) -> Self {
        for (opt, selected) in options {
            self.defaults.push(selected);
            self.options.push(opt);
        }
        self
    }

    /// Add simple string options (all unselected)
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self
    where
        T: ToString + Clone,
    {
        for item in items {
            let label = item.to_string();
            self.defaults.push(false);
            self.options.push(SelectOption::new(item, label));
        }
        self
    }

    /// Set minimum required selections
    pub fn min(mut self, min: usize) -> Self {
        self.min_selections = Some(min);
        self
    }

    /// Set maximum allowed selections
    pub fn max(mut self, max: usize) -> Self {
        self.max_selections = Some(max);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set color mode
    pub fn color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Run the prompt
    pub fn prompt(self) -> PromptResult<Vec<T>>
    where
        T: Clone,
    {
        if self.options.is_empty() {
            return Err(PromptError::ValidationFailed(
                "No options provided".to_string(),
            ));
        }

        if !is_interactive() {
            // Return default selections
            let selected: Vec<T> = self
                .options
                .iter()
                .zip(&self.defaults)
                .filter(|(_, &selected)| selected)
                .map(|(opt, _)| opt.value.clone())
                .collect();
            return Ok(selected);
        }

        terminal::enable_raw_mode()?;
        let result = self.run_interactive();
        terminal::disable_raw_mode()?;

        result
    }

    fn run_interactive(self) -> PromptResult<Vec<T>>
    where
        T: Clone,
    {
        let mut stdout = io::stdout();
        let mut cursor = 0usize;
        let mut selections = self.defaults.clone();
        let mut scroll_offset = 0;

        // Print prompt message
        let prefix = Styled::new(&self.theme.prompt_prefix)
            .with_color_mode(self.color_mode)
            .fg(self.theme.active_color);

        let hint = Styled::new("(Space to select, Enter to confirm)")
            .with_color_mode(self.color_mode)
            .fg(self.theme.hint_color);

        println!("{prefix} {} {hint}", self.message);

        loop {
            let visible_end = (scroll_offset + self.page_size).min(self.options.len());

            // Render options
            for (i, option) in self.options[scroll_offset..visible_end].iter().enumerate() {
                let idx = scroll_offset + i;
                let is_cursor = idx == cursor;
                let is_selected = selections[idx];

                let check = if is_selected {
                    Styled::new(&self.theme.selected_prefix)
                        .with_color_mode(self.color_mode)
                        .fg(self.theme.active_color)
                } else {
                    Styled::new(&self.theme.unselected_prefix)
                        .with_color_mode(self.color_mode)
                };

                let cursor_indicator = if is_cursor { "❯" } else { " " };

                let label = if is_cursor {
                    Styled::new(&option.label)
                        .with_color_mode(self.color_mode)
                        .fg(self.theme.active_color)
                } else {
                    Styled::new(&option.label).with_color_mode(self.color_mode)
                };

                println!("{cursor_indicator} {check} {label}");
            }

            // Handle input
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if cursor > 0 {
                            cursor -= 1;
                            if cursor < scroll_offset {
                                scroll_offset = cursor;
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if cursor < self.options.len() - 1 {
                            cursor += 1;
                            if cursor >= scroll_offset + self.page_size {
                                scroll_offset = cursor - self.page_size + 1;
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        // Toggle selection
                        let current = selections[cursor];
                        let selected_count: usize = selections.iter().filter(|&&s| s).count();

                        // Check max before selecting
                        if !current {
                            if let Some(max) = self.max_selections {
                                if selected_count >= max {
                                    continue;
                                }
                            }
                        }

                        selections[cursor] = !current;
                    }
                    KeyCode::Char('a') => {
                        // Select/deselect all
                        let all_selected = selections.iter().all(|&s| s);
                        for s in &mut selections {
                            *s = !all_selected;
                        }
                    }
                    KeyCode::Enter => {
                        let selected_count: usize =
                            selections.iter().filter(|&&s| s).count();

                        // Validate minimum
                        if let Some(min) = self.min_selections {
                            if selected_count < min {
                                // Show error but don't exit
                                continue;
                            }
                        }

                        // Clear display
                        execute!(
                            stdout,
                            cursor::MoveUp(visible_end as u16 - scroll_offset as u16),
                            terminal::Clear(ClearType::FromCursorDown)
                        )?;

                        // Collect selected values
                        let result: Vec<T> = self
                            .options
                            .iter()
                            .zip(&selections)
                            .filter(|(_, &selected)| selected)
                            .map(|(opt, _)| opt.value.clone())
                            .collect();

                        // Show selected count
                        let count_styled = Styled::new(format!("{} selected", result.len()))
                            .with_color_mode(self.color_mode)
                            .fg(self.theme.active_color);
                        println!("{count_styled}");

                        return Ok(result);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        execute!(
                            stdout,
                            cursor::MoveUp(visible_end as u16 - scroll_offset as u16),
                            terminal::Clear(ClearType::FromCursorDown)
                        )?;
                        return Err(PromptError::Cancelled);
                    }
                    _ => {}
                }
            }

            // Move cursor up to redraw
            execute!(
                stdout,
                cursor::MoveUp(visible_end as u16 - scroll_offset as u16),
                terminal::Clear(ClearType::FromCursorDown)
            )?;
        }
    }
}

/// Convenience function for multi-selection
pub fn multiselect<T: Clone + ToString>(message: &str, items: Vec<T>) -> PromptResult<Vec<T>> {
    MultiSelect::new(message).items(items).prompt()
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Note: Interactive prompts are difficult to unit test
    // These tests verify non-interactive behavior

    #[test]
    fn test_confirm_non_interactive_with_default() {
        // When not interactive and default is set, return default
        let confirm = Confirm::new("Test?").default(true);
        // This would return true if run non-interactively
    }

    #[test]
    fn test_select_option_creation() {
        let opt = SelectOption::new("value", "Label")
            .with_hint("Some hint");
        assert_eq!(opt.label, "Label");
        assert_eq!(opt.hint, Some("Some hint".to_string()));
    }

    #[test]
    fn test_theme_ascii() {
        let theme = Theme::ascii();
        assert_eq!(theme.selected_prefix, "[x]");
        assert_eq!(theme.unselected_prefix, "[ ]");
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **081-cli-color.md**: Color support
- **082-cli-progress.md**: Progress indicators
- **088-cli-init-scaffold.md**: Uses prompts for project initialization
