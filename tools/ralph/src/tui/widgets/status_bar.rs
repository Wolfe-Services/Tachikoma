//! Status bar widget

use ratatui::{
    prelude::*,
    widgets::Paragraph,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::tui::app::App;

/// Widget for displaying keyboard shortcuts and status
pub struct StatusBarWidget;

impl StatusBarWidget {
    /// Render the status bar
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let shortcuts = vec![
            ("p", if app.is_paused { "resume" } else { "pause" }),
            ("d", "dashboard"),
            ("q", "quit"),
            ("?", "help"),
        ];

        let mut spans: Vec<Span> = Vec::new();
        
        for (key, action) in shortcuts {
            spans.push(Span::styled(
                format!("[{}]", key),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            ));
            spans.push(Span::raw(format!("{} ", action)));
        }

        // Add pause indicator if paused
        if app.is_paused {
            spans.push(Span::styled(
                " ⏸ PAUSED ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            ));
        }

        // Add running indicator
        if app.is_running && !app.is_paused {
            spans.push(Span::styled(
                " ● RUNNING ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
            ));
        }

        // Add cost display on the right
        let cost_str = format!(" ${:.2} ", app.total_cost);
        
        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        
        frame.render_widget(paragraph, area);
        
        // Render cost on the right side
        let cost_area = Rect::new(
            area.x + area.width - cost_str.len() as u16 - 1,
            area.y,
            cost_str.len() as u16,
            1
        );
        
        let cost_widget = Paragraph::new(cost_str)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(cost_widget, cost_area);
    }
}
