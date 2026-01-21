//! Progress bar widget

use ratatui::{
    prelude::*,
    widgets::{Gauge, Block, Borders},
    style::{Color, Style},
};

use crate::tui::app::App;

/// Widget for displaying spec completion progress
pub struct ProgressBarWidget;

impl ProgressBarWidget {
    /// Render the progress bar
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let progress = app.progress_percentage() / 100.0;
        let label = format!(
            "{}/{} specs ({:.0}%)",
            app.specs_completed,
            app.specs_total,
            app.progress_percentage()
        );

        let color = if progress >= 0.9 {
            Color::Green
        } else if progress >= 0.5 {
            Color::Yellow
        } else {
            Color::Blue
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
            .ratio(progress.min(1.0))
            .label(label);

        frame.render_widget(gauge, area);
    }
}
