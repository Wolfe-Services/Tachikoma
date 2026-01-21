//! UI rendering for the TUI

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::app::{App, View};
use super::widgets::{
    TaskListWidget, OutputPanelWidget, ProgressBarWidget, 
    TokenGaugeWidget, StatusBarWidget
};

/// Main UI renderer
pub struct Ui;

impl Ui {
    /// Render the UI based on current view
    pub fn render(frame: &mut Frame, app: &App) {
        match app.current_view {
            View::Split => Self::render_split_view(frame, app),
            View::Dashboard => Self::render_dashboard_view(frame, app),
            View::Help => Self::render_help_view(frame, app),
        }
    }

    /// Render the default split view
    fn render_split_view(frame: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),     // Main area
                Constraint::Length(2),   // Progress + tokens
                Constraint::Length(1),   // Status bar
            ])
            .split(frame.area());

        // Split main area into left (tasks) and right (output)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(32),  // Tasks panel (fixed width)
                Constraint::Min(40),     // Output panel (flexible)
            ])
            .split(chunks[0]);

        // Render task list
        TaskListWidget::render(frame, main_chunks[0], app);

        // Render output panel
        OutputPanelWidget::render(frame, main_chunks[1], app);

        // Render progress and token gauges
        let gauge_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        ProgressBarWidget::render(frame, gauge_chunks[0], app);
        TokenGaugeWidget::render(frame, gauge_chunks[1], app);

        // Render status bar
        StatusBarWidget::render(frame, chunks[2], app);
    }

    /// Render the dashboard view
    fn render_dashboard_view(frame: &mut Frame, app: &App) {
        let area = frame.area();
        
        let block = Block::default()
            .title(" Session Dashboard ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Progress row
                Constraint::Length(1),  // Separator
                Constraint::Length(8),  // Stats
                Constraint::Length(1),  // Separator
                Constraint::Length(3),  // Token usage
                Constraint::Min(1),     // Spacer
            ])
            .split(inner);

        // Progress row
        let progress_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);

        let specs_text = format!(
            "Specs: {}/{} ({:.1}%)",
            app.specs_completed,
            app.specs_total,
            app.progress_percentage()
        );
        let criteria_text = format!(
            "Criteria: {}/{} ({:.1}%)",
            app.criteria_completed,
            app.criteria_total,
            if app.criteria_total > 0 {
                (app.criteria_completed as f64 / app.criteria_total as f64) * 100.0
            } else {
                0.0
            }
        );

        frame.render_widget(
            Paragraph::new(specs_text)
                .style(Style::default().add_modifier(Modifier::BOLD)),
            progress_chunks[0]
        );
        frame.render_widget(
            Paragraph::new(criteria_text)
                .style(Style::default().add_modifier(Modifier::BOLD)),
            progress_chunks[1]
        );

        // Stats section
        let stats_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[2]);

        let session_stats = vec![
            Line::from(Span::styled("Session Stats", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("─────────────"),
            Line::from(format!("Started: {} ago", app.session_duration())),
            Line::from(format!("Iterations: {}", app.iterations)),
            Line::from(format!("Reboots: {}", app.reboots)),
            Line::from(format!("Commits: {}", app.commits)),
        ];

        // Estimate costs
        let input_cost = (app.input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (app.output_tokens as f64 / 1_000_000.0) * 15.0;
        
        let cost_stats = vec![
            Line::from(Span::styled("Cost Breakdown", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("──────────────"),
            Line::from(format!("Input:  ${:.2}", input_cost)),
            Line::from(format!("Output: ${:.2}", output_cost)),
            Line::from(format!("Total:  ${:.2}", app.total_cost)),
            Line::from(""),
        ];

        frame.render_widget(Paragraph::new(session_stats), stats_chunks[0]);
        frame.render_widget(Paragraph::new(cost_stats), stats_chunks[1]);

        // Token usage with gauge
        let token_label = format!(
            "Token Usage: {}k / {}k",
            app.total_tokens() / 1000,
            app.redline_threshold / 1000
        );
        
        TokenGaugeWidget::render(frame, chunks[4], app);

        // Hint to return
        let hint = Paragraph::new("Press 'd' or ESC to return")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[5]);
    }

    /// Render the help view
    fn render_help_view(frame: &mut Frame, app: &App) {
        let area = centered_rect(60, 70, frame.area());
        
        frame.render_widget(Clear, area);
        
        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);

        let help_text = vec![
            Line::from(Span::styled("Keyboard Controls", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  p      ", Style::default().fg(Color::Cyan)),
                Span::raw("Pause/Resume execution"),
            ]),
            Line::from(vec![
                Span::styled("  d      ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle dashboard view"),
            ]),
            Line::from(vec![
                Span::styled("  i      ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle iteration history"),
            ]),
            Line::from(vec![
                Span::styled("  l      ", Style::default().fg(Color::Cyan)),
                Span::raw("Show full log (scroll to top)"),
            ]),
            Line::from(vec![
                Span::styled("  q      ", Style::default().fg(Color::Cyan)),
                Span::raw("Quit (with confirmation if running)"),
            ]),
            Line::from(vec![
                Span::styled("  ?      ", Style::default().fg(Color::Cyan)),
                Span::raw("Show this help"),
            ]),
            Line::from(""),
            Line::from(Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ↑/↓    ", Style::default().fg(Color::Cyan)),
                Span::raw("Scroll in focused pane"),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/Dn", Style::default().fg(Color::Cyan)),
                Span::raw("Page scroll in output"),
            ]),
            Line::from(vec![
                Span::styled("  Tab    ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch focus between panes"),
            ]),
            Line::from(vec![
                Span::styled("  Home   ", Style::default().fg(Color::Cyan)),
                Span::raw("Scroll to top of output"),
            ]),
            Line::from(vec![
                Span::styled("  End    ", Style::default().fg(Color::Cyan)),
                Span::raw("Scroll to bottom of output"),
            ]),
            Line::from(""),
            Line::from("Press any key to close help..."),
        ];

        let paragraph = Paragraph::new(help_text)
            .wrap(Wrap { trim: false });
        
        frame.render_widget(paragraph, inner);
    }
}

/// Create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
