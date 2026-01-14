//! Task list widget

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Color, Modifier, Style},
};

use crate::tui::app::{App, FocusPane, Task, TaskStatus};

/// Widget for displaying the task/spec list
pub struct TaskListWidget;

impl TaskListWidget {
    /// Render the task list
    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let is_focused = app.focus_pane == FocusPane::Tasks;
        
        let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };
        
        let block = Block::default()
            .title(" Tasks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let items: Vec<ListItem> = app.tasks.iter().enumerate().map(|(idx, task)| {
            let (icon, style) = match task.status {
                TaskStatus::Completed => ("✓", Style::default().fg(Color::Green)),
                TaskStatus::InProgress => ("→", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                TaskStatus::Failed => ("✗", Style::default().fg(Color::Red)),
                TaskStatus::Pending => ("○", Style::default().fg(Color::DarkGray)),
            };
            
            let progress = if task.criteria_total > 0 {
                format!(" ({}/{})", task.criteria_done, task.criteria_total)
            } else {
                String::new()
            };
            
            let content = format!("{} {:03} {}{}", icon, task.id, task.name, progress);
            let item = ListItem::new(content);
            
            if idx == app.selected_task {
                item.style(style.add_modifier(Modifier::REVERSED))
            } else {
                item.style(style)
            }
        }).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut state = ListState::default();
        state.select(Some(app.selected_task));
        
        frame.render_stateful_widget(list, area, &mut state);
    }
}
