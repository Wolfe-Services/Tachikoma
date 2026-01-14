//! Output panel widget

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    style::{Color, Style},
    text::{Line, Span},
};

use crate::tui::app::{App, FocusPane, OutputLevel};

/// Widget for displaying the agent output
pub struct OutputPanelWidget;

impl OutputPanelWidget {
    /// Render the output panel
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let is_focused = app.focus_pane == FocusPane::Output;
        
        let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };
        
        let block = Block::default()
            .title(" Output ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner_area = block.inner(area);
        let visible_height = inner_area.height as usize;
        
        // Calculate scroll
        let total_lines = app.output_lines.len();
        let start = if total_lines > visible_height {
            app.output_scroll.min(total_lines - visible_height)
        } else {
            0
        };
        
        let lines: Vec<Line> = app.output_lines
            .iter()
            .skip(start)
            .take(visible_height)
            .map(|line| {
                let time_str = line.timestamp.format("%H:%M:%S").to_string();
                let time_span = Span::styled(
                    format!("[{}] ", time_str),
                    Style::default().fg(Color::DarkGray)
                );
                
                let (text_color, prefix) = match line.level {
                    OutputLevel::Info => (Color::White, ""),
                    OutputLevel::Debug => (Color::DarkGray, ""),
                    OutputLevel::Tool => (Color::Cyan, ""),
                    OutputLevel::ToolResult => (Color::Blue, ""),
                    OutputLevel::Error => (Color::Red, "✗ "),
                    OutputLevel::Success => (Color::Green, ""),
                    OutputLevel::Text => (Color::White, ""),
                };
                
                let text_span = Span::styled(
                    format!("{}{}", prefix, &line.text),
                    Style::default().fg(text_color)
                );
                
                Line::from(vec![time_span, text_span])
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
        
        // Scrollbar hint
        if total_lines > visible_height {
            let scroll_pct = if total_lines - visible_height > 0 {
                (start as f64 / (total_lines - visible_height) as f64 * 100.0) as u16
            } else {
                100
            };
            
            let scroll_hint = format!(" {}% ", scroll_pct);
            let hint_area = Rect::new(
                area.x + area.width - scroll_hint.len() as u16 - 2,
                area.y,
                scroll_hint.len() as u16,
                1
            );
            
            let hint = Paragraph::new(scroll_hint)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(hint, hint_area);
        }
    }
}
