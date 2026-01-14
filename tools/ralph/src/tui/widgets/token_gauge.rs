//! Token gauge widget

use ratatui::{
    prelude::*,
    widgets::{Gauge, Block, Borders},
    style::{Color, Modifier, Style},
};

use crate::tui::app::App;

/// Widget for displaying token usage
pub struct TokenGaugeWidget;

impl TokenGaugeWidget {
    /// Render the token gauge
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let usage = app.token_percentage() / 100.0;
        let is_redline = app.is_redline();
        
        // Format token counts with K suffix
        let total_k = app.total_tokens() as f64 / 1000.0;
        let limit_k = app.redline_threshold as f64 / 1000.0;
        
        let label = format!("Tokens: {:.0}k/{:.0}k", total_k, limit_k);

        // Color based on usage
        let color = if is_redline {
            Color::Red
        } else if usage >= 0.8 {
            Color::LightRed
        } else if usage >= 0.6 {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let mut style = Style::default().fg(color).bg(Color::DarkGray);
        if is_redline {
            style = style.add_modifier(Modifier::RAPID_BLINK);
        }

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(style)
            .ratio(usage.min(1.0))
            .label(label);

        frame.render_widget(gauge, area);
    }
}
