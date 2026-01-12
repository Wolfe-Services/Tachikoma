//! Table formatting for CLI output.

use serde::Serialize;
use serde_json::Value;

/// Table rendering style
#[derive(Debug, Clone, Copy, Default)]
pub enum TableStyle {
    #[default]
    Plain,
    Bordered,
    Markdown,
    Compact,
}

/// Column alignment
#[derive(Debug, Clone, Copy, Default)]
pub enum Alignment {
    #[default]
    Left,
    Right,
    Center,
}

/// Table column definition
#[derive(Debug, Clone)]
pub struct Column {
    pub header: String,
    pub alignment: Alignment,
    pub min_width: usize,
    pub max_width: Option<usize>,
    pub color: Option<&'static str>,
}

impl Column {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            alignment: Alignment::Left,
            min_width: 0,
            max_width: None,
            color: None,
        }
    }

    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    pub fn max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn color(mut self, color: &'static str) -> Self {
        self.color = Some(color);
        self
    }
}

/// Table structure
#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    style: TableStyle,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            style: TableStyle::default(),
        }
    }

    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    pub fn add_row(&mut self, row: Vec<impl Into<String>>) {
        self.rows.push(row.into_iter().map(Into::into).collect());
    }

    pub fn add_rows(&mut self, rows: impl IntoIterator<Item = Vec<impl Into<String>>>) {
        for row in rows {
            self.add_row(row);
        }
    }

    /// Calculate column widths
    fn calculate_widths(&self, max_total: usize) -> Vec<usize> {
        let mut widths: Vec<usize> = self
            .columns
            .iter()
            .map(|c| c.header.len().max(c.min_width))
            .collect();

        // Consider row content
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Apply max widths
        for (i, col) in self.columns.iter().enumerate() {
            if let Some(max) = col.max_width {
                widths[i] = widths[i].min(max);
            }
        }

        // Ensure fits in terminal
        let separator_width = if self.columns.len() > 1 { (self.columns.len() - 1) * 3 } else { 0 };
        let total: usize = widths.iter().sum::<usize>() + separator_width;
        if total > max_total && widths.len() > 1 {
            let excess = total - max_total;
            let reduce_per = excess / widths.len() + 1;
            for w in &mut widths {
                *w = (*w).saturating_sub(reduce_per).max(5);
            }
        }

        widths
    }

    /// Render the table to a string
    pub fn render(&self, max_width: usize, color: bool) -> String {
        let widths = self.calculate_widths(max_width);
        let mut output = String::new();

        match self.style {
            TableStyle::Plain => self.render_plain(&mut output, &widths, color),
            TableStyle::Bordered => self.render_bordered(&mut output, &widths, color),
            TableStyle::Markdown => self.render_markdown(&mut output, &widths),
            TableStyle::Compact => self.render_compact(&mut output, &widths, color),
        }

        output
    }

    fn render_plain(&self, output: &mut String, widths: &[usize], color: bool) {
        // Header
        let header: Vec<_> = self
            .columns
            .iter()
            .zip(widths)
            .map(|(col, &w)| self.format_cell(&col.header, w, col.alignment))
            .collect();

        if color {
            output.push_str("\x1b[1m");
        }
        output.push_str(&header.join("   "));
        if color {
            output.push_str("\x1b[0m");
        }
        output.push('\n');

        // Separator
        let sep: Vec<_> = widths.iter().map(|&w| "-".repeat(w)).collect();
        output.push_str(&sep.join("   "));
        output.push('\n');

        // Rows
        for row in &self.rows {
            let cells: Vec<_> = row
                .iter()
                .zip(&self.columns)
                .zip(widths)
                .map(|((cell, col), &w)| self.format_cell(cell, w, col.alignment))
                .collect();
            output.push_str(&cells.join("   "));
            output.push('\n');
        }
    }

    fn render_bordered(&self, output: &mut String, widths: &[usize], color: bool) {
        let horiz = |output: &mut String, left: &str, mid: &str, right: &str| {
            output.push_str(left);
            let parts: Vec<_> = widths.iter().map(|&w| "─".repeat(w + 2)).collect();
            output.push_str(&parts.join(mid));
            output.push_str(right);
            output.push('\n');
        };

        // Top border
        horiz(output, "┌", "┬", "┐");

        // Header
        output.push_str("│");
        for (i, col) in self.columns.iter().enumerate() {
            let cell = self.format_cell(&col.header, widths[i], col.alignment);
            if color {
                output.push_str(&format!(" \x1b[1m{cell}\x1b[0m │"));
            } else {
                output.push_str(&format!(" {cell} │"));
            }
        }
        output.push('\n');

        // Header separator
        horiz(output, "├", "┼", "┤");

        // Rows
        for row in &self.rows {
            output.push_str("│");
            for (i, cell) in row.iter().enumerate() {
                if i >= self.columns.len() {
                    break;
                }
                let col = &self.columns[i];
                let formatted = self.format_cell(cell, widths[i], col.alignment);
                output.push_str(&format!(" {formatted} │"));
            }
            output.push('\n');
        }

        // Bottom border
        horiz(output, "└", "┴", "┘");
    }

    fn render_markdown(&self, output: &mut String, widths: &[usize]) {
        // Header
        output.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            let cell = self.format_cell(&col.header, widths[i], col.alignment);
            output.push_str(&format!(" {cell} |"));
        }
        output.push('\n');

        // Separator with alignment
        output.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            let w = widths[i];
            let sep = match col.alignment {
                Alignment::Left => format!(":{}", "-".repeat(w)),
                Alignment::Right => format!("{}:", "-".repeat(w)),
                Alignment::Center => format!(":{}:", "-".repeat(w.saturating_sub(1))),
            };
            output.push_str(&format!(" {sep} |"));
        }
        output.push('\n');

        // Rows
        for row in &self.rows {
            output.push('|');
            for (i, cell) in row.iter().enumerate() {
                if i >= self.columns.len() {
                    break;
                }
                let col = &self.columns[i];
                let formatted = self.format_cell(cell, widths[i], col.alignment);
                output.push_str(&format!(" {formatted} |"));
            }
            output.push('\n');
        }
    }

    fn render_compact(&self, output: &mut String, widths: &[usize], _color: bool) {
        for row in &self.rows {
            let cells: Vec<_> = row
                .iter()
                .zip(&self.columns)
                .zip(widths)
                .map(|((cell, col), &w)| self.format_cell(cell, w, col.alignment))
                .collect();
            output.push_str(&cells.join(" "));
            output.push('\n');
        }
    }

    fn format_cell(&self, content: &str, width: usize, alignment: Alignment) -> String {
        let content = if content.len() > width {
            format!("{}...", &content[..width.saturating_sub(3)])
        } else {
            content.to_string()
        };

        match alignment {
            Alignment::Left => format!("{content:<width$}"),
            Alignment::Right => format!("{content:>width$}"),
            Alignment::Center => format!("{content:^width$}"),
        }
    }

    /// Convert to JSON-serializable rows
    pub fn to_json_rows(&self) -> Vec<serde_json::Map<String, Value>> {
        self.rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, cell) in row.iter().enumerate() {
                    if i < self.columns.len() {
                        map.insert(
                            self.columns[i].header.clone(),
                            Value::String(cell.clone()),
                        );
                    }
                }
                map
            })
            .collect()
    }
}

/// Builder for creating tables
pub struct TableBuilder {
    columns: Vec<Column>,
    style: TableStyle,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            style: TableStyle::default(),
        }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn columns(mut self, columns: impl IntoIterator<Item = Column>) -> Self {
        self.columns.extend(columns);
        self
    }

    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    pub fn build(self) -> Table {
        Table::new(self.columns).style(self.style)
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_plain() {
        let mut table = Table::new(vec![
            Column::new("Name"),
            Column::new("Value").align(Alignment::Right),
        ]);
        table.add_row(vec!["foo", "123"]);
        table.add_row(vec!["bar", "456"]);

        let output = table.render(80, false);
        assert!(output.contains("Name"));
        assert!(output.contains("foo"));
        assert!(output.contains("123"));
        assert!(output.contains("---"));
    }

    #[test]
    fn test_table_markdown() {
        let mut table = Table::new(vec![
            Column::new("A"),
            Column::new("B"),
        ]).style(TableStyle::Markdown);
        table.add_row(vec!["1", "2"]);

        let output = table.render(80, false);
        assert!(output.contains("|"));
        assert!(output.contains("---"));
    }

    #[test]
    fn test_table_json_rows() {
        let mut table = Table::new(vec![
            Column::new("name"),
            Column::new("value"),
        ]);
        table.add_row(vec!["test", "123"]);

        let rows = table.to_json_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name").unwrap(), "test");
    }

    #[test]
    fn test_column_width_constraints() {
        let mut table = Table::new(vec![
            Column::new("Short").min_width(10),
            Column::new("Long").max_width(5),
        ]);
        table.add_row(vec!["a", "very long text"]);

        let widths = table.calculate_widths(50);
        assert!(widths[0] >= 10); // Respects min_width
        assert!(widths[1] <= 5);  // Respects max_width
    }

    #[test]
    fn test_table_builder() {
        let table = TableBuilder::new()
            .column(Column::new("Name"))
            .column(Column::new("Value"))
            .style(TableStyle::Bordered)
            .build();

        assert_eq!(table.columns.len(), 2);
        assert!(matches!(table.style, TableStyle::Bordered));
    }
}