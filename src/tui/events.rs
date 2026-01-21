//! Keyboard event handling for the TUI

use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use anyhow::Result;

use super::app::App;

/// Event handler for keyboard input
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for events with timeout
    pub fn poll(&self) -> Result<Option<KeyEvent>> {
        if event::poll(self.tick_rate)? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    /// Handle a key event, returns true if handled
    pub fn handle_key(&self, app: &mut App, key: KeyEvent) -> bool {
        // Global keybindings (work in any view)
        match key.code {
            KeyCode::Char('q') => {
                app.request_quit();
                return true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
                return true;
            }
            _ => {}
        }

        // View-specific keybindings
        match app.current_view {
            super::app::View::Help => {
                // Any key exits help
                app.current_view = super::app::View::Split;
                true
            }
            super::app::View::Dashboard => {
                match key.code {
                    KeyCode::Char('d') | KeyCode::Esc => {
                        app.current_view = super::app::View::Split;
                    }
                    KeyCode::Char('?') => {
                        app.show_help();
                    }
                    _ => return false,
                }
                true
            }
            super::app::View::Split => {
                match key.code {
                    // Pause/Resume
                    KeyCode::Char('p') => {
                        app.toggle_pause();
                    }
                    // Toggle dashboard
                    KeyCode::Char('d') => {
                        app.toggle_view();
                    }
                    // Toggle iteration history (same as dashboard for now)
                    KeyCode::Char('i') => {
                        app.toggle_view();
                    }
                    // Show full log (scroll to top)
                    KeyCode::Char('l') => {
                        app.output_scroll = 0;
                    }
                    // Help
                    KeyCode::Char('?') => {
                        app.show_help();
                    }
                    // Navigation
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.scroll_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.scroll_down();
                    }
                    KeyCode::PageUp => {
                        app.page_up();
                    }
                    KeyCode::PageDown => {
                        app.page_down();
                    }
                    // Switch focus between panes
                    KeyCode::Tab => {
                        app.toggle_focus();
                    }
                    // Home/End for output
                    KeyCode::Home => {
                        app.output_scroll = 0;
                    }
                    KeyCode::End => {
                        app.output_scroll = app.output_lines.len().saturating_sub(1);
                    }
                    _ => return false,
                }
                true
            }
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(100)
    }
}
