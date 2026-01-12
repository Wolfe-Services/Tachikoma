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
pub use select::{Select, SelectOption};

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