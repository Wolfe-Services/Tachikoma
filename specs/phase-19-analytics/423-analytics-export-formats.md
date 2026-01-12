# Spec 423: Export Formats

## Phase
19 - Analytics/Telemetry

## Spec ID
423

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 411: Analytics Export (export system)
- Spec 421: Analytics Reports (report generation)

## Estimated Context
~8%

---

## Objective

Implement comprehensive export format support for analytics data, enabling users to export data in various formats for external analysis, archival, and integration with other tools.

---

## Acceptance Criteria

- [ ] Support JSON export with schema
- [ ] Implement CSV export with proper escaping
- [ ] Create NDJSON streaming export
- [ ] Support Parquet columnar format
- [ ] Implement PDF report export
- [ ] Support HTML report export
- [ ] Enable Excel export
- [ ] Provide format conversion utilities

---

## Implementation Details

### Export Formats

```rust
// src/analytics/export_formats.rs

use crate::analytics::reports::{
    ChartSection, ChartType, Report, ReportSection, TableSection,
};
use crate::analytics::types::AnalyticsEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
    Ndjson,
    Parquet,
    Html,
    Markdown,
    Pdf,
    Excel,
}

impl ExportFormat {
    pub fn file_extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Ndjson => "ndjson",
            Self::Parquet => "parquet",
            Self::Html => "html",
            Self::Markdown => "md",
            Self::Pdf => "pdf",
            Self::Excel => "xlsx",
        }
    }

    pub fn content_type(&self) -> &str {
        match self {
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Ndjson => "application/x-ndjson",
            Self::Parquet => "application/vnd.apache.parquet",
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
            Self::Pdf => "application/pdf",
            Self::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        }
    }
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatOptions {
    /// Pretty print JSON
    pub pretty: bool,
    /// Include headers in CSV
    pub include_headers: bool,
    /// CSV delimiter
    pub csv_delimiter: char,
    /// Include schema information
    pub include_schema: bool,
    /// Timestamp format
    pub timestamp_format: String,
    /// Null value representation
    pub null_value: String,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            include_headers: true,
            csv_delimiter: ',',
            include_schema: false,
            timestamp_format: "%Y-%m-%dT%H:%M:%SZ".to_string(),
            null_value: "".to_string(),
        }
    }
}

/// Format writer trait
pub trait FormatWriter {
    fn write_events(&self, events: &[AnalyticsEvent], output: &mut dyn Write) -> Result<(), FormatError>;
    fn write_report(&self, report: &Report, output: &mut dyn Write) -> Result<(), FormatError>;
}

/// JSON format writer
pub struct JsonWriter {
    options: FormatOptions,
}

impl JsonWriter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }
}

impl FormatWriter for JsonWriter {
    fn write_events(&self, events: &[AnalyticsEvent], output: &mut dyn Write) -> Result<(), FormatError> {
        let json = if self.options.pretty {
            serde_json::to_string_pretty(events)
        } else {
            serde_json::to_string(events)
        }
        .map_err(|e| FormatError::Serialization(e.to_string()))?;

        output
            .write_all(json.as_bytes())
            .map_err(|e| FormatError::Io(e.to_string()))?;

        Ok(())
    }

    fn write_report(&self, report: &Report, output: &mut dyn Write) -> Result<(), FormatError> {
        let json = if self.options.pretty {
            serde_json::to_string_pretty(report)
        } else {
            serde_json::to_string(report)
        }
        .map_err(|e| FormatError::Serialization(e.to_string()))?;

        output
            .write_all(json.as_bytes())
            .map_err(|e| FormatError::Io(e.to_string()))?;

        Ok(())
    }
}

/// CSV format writer
pub struct CsvWriter {
    options: FormatOptions,
}

impl CsvWriter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    fn escape_csv_field(&self, field: &str) -> String {
        let needs_quotes = field.contains(self.options.csv_delimiter)
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r');

        if needs_quotes {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

impl FormatWriter for CsvWriter {
    fn write_events(&self, events: &[AnalyticsEvent], output: &mut dyn Write) -> Result<(), FormatError> {
        let delimiter = self.options.csv_delimiter;

        // Write headers
        if self.options.include_headers {
            let headers = vec![
                "id", "category", "event_type", "timestamp", "session_id",
                "priority", "data",
            ];
            writeln!(output, "{}", headers.join(&delimiter.to_string()))
                .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        // Write rows
        for event in events {
            let row = vec![
                format!("{:?}", event.id),
                format!("{:?}", event.category),
                format!("{:?}", event.event_type),
                event.timestamp.format(&self.options.timestamp_format).to_string(),
                event.session_id.map(|id| id.to_string()).unwrap_or_default(),
                format!("{:?}", event.priority),
                self.escape_csv_field(&serde_json::to_string(&event.data).unwrap_or_default()),
            ];

            writeln!(output, "{}", row.join(&delimiter.to_string()))
                .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        Ok(())
    }

    fn write_report(&self, report: &Report, output: &mut dyn Write) -> Result<(), FormatError> {
        // For reports, extract table sections and write as CSV
        for section in &report.sections {
            if let ReportSection::Table(table) = section {
                self.write_table(table, output)?;
            }
        }

        Ok(())
    }
}

impl CsvWriter {
    fn write_table(&self, table: &TableSection, output: &mut dyn Write) -> Result<(), FormatError> {
        let delimiter = self.options.csv_delimiter;

        // Write title as comment
        writeln!(output, "# {}", table.title)
            .map_err(|e| FormatError::Io(e.to_string()))?;

        // Write headers
        if self.options.include_headers {
            writeln!(output, "{}", table.headers.join(&delimiter.to_string()))
                .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        // Write rows
        for row in &table.rows {
            let escaped: Vec<String> = row.iter().map(|f| self.escape_csv_field(f)).collect();
            writeln!(output, "{}", escaped.join(&delimiter.to_string()))
                .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        Ok(())
    }
}

/// NDJSON (Newline Delimited JSON) writer
pub struct NdjsonWriter {
    options: FormatOptions,
}

impl NdjsonWriter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }
}

impl FormatWriter for NdjsonWriter {
    fn write_events(&self, events: &[AnalyticsEvent], output: &mut dyn Write) -> Result<(), FormatError> {
        for event in events {
            let json = serde_json::to_string(event)
                .map_err(|e| FormatError::Serialization(e.to_string()))?;

            writeln!(output, "{}", json)
                .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        Ok(())
    }

    fn write_report(&self, report: &Report, output: &mut dyn Write) -> Result<(), FormatError> {
        // Write report metadata first
        let meta = serde_json::json!({
            "type": "report_metadata",
            "id": report.id,
            "title": report.title,
            "generated_at": report.generated_at,
        });

        writeln!(output, "{}", serde_json::to_string(&meta).unwrap())
            .map_err(|e| FormatError::Io(e.to_string()))?;

        // Write each section
        for section in &report.sections {
            let json = serde_json::to_string(section)
                .map_err(|e| FormatError::Serialization(e.to_string()))?;
            writeln!(output, "{}", json)
                .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        Ok(())
    }
}

/// HTML report writer
pub struct HtmlWriter {
    options: FormatOptions,
}

impl HtmlWriter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}

impl FormatWriter for HtmlWriter {
    fn write_events(&self, events: &[AnalyticsEvent], output: &mut dyn Write) -> Result<(), FormatError> {
        write!(
            output,
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Analytics Events Export</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f5f5f5; }}
        tr:nth-child(even) {{ background-color: #fafafa; }}
    </style>
</head>
<body>
    <h1>Analytics Events</h1>
    <p>Exported: {}</p>
    <table>
        <thead>
            <tr>
                <th>ID</th>
                <th>Category</th>
                <th>Event Type</th>
                <th>Timestamp</th>
                <th>Priority</th>
            </tr>
        </thead>
        <tbody>
"#,
            Utc::now().format(&self.options.timestamp_format)
        )
        .map_err(|e| FormatError::Io(e.to_string()))?;

        for event in events {
            write!(
                output,
                r#"            <tr>
                <td>{}</td>
                <td>{:?}</td>
                <td>{:?}</td>
                <td>{}</td>
                <td>{:?}</td>
            </tr>
"#,
                self.escape_html(&format!("{:?}", event.id)),
                event.category,
                event.event_type,
                event.timestamp.format(&self.options.timestamp_format),
                event.priority
            )
            .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        write!(
            output,
            r#"        </tbody>
    </table>
</body>
</html>"#
        )
        .map_err(|e| FormatError::Io(e.to_string()))?;

        Ok(())
    }

    fn write_report(&self, report: &Report, output: &mut dyn Write) -> Result<(), FormatError> {
        // Write HTML header
        write!(
            output,
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; max-width: 1200px; margin: 0 auto; padding: 20px; }}
        h1 {{ color: #333; border-bottom: 2px solid #007bff; padding-bottom: 10px; }}
        h2 {{ color: #555; margin-top: 30px; }}
        .section {{ margin-bottom: 30px; padding: 20px; background: #f9f9f9; border-radius: 8px; }}
        .metric {{ display: inline-block; margin: 10px; padding: 15px; background: white; border-radius: 4px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #007bff; }}
        .metric-name {{ color: #666; font-size: 14px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 10px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 10px; text-align: left; }}
        th {{ background-color: #f5f5f5; }}
        .footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>Period: {} to {}</p>
    <p>Generated: {}</p>
"#,
            self.escape_html(&report.title),
            self.escape_html(&report.title),
            report.period.start.format("%Y-%m-%d %H:%M"),
            report.period.end.format("%Y-%m-%d %H:%M"),
            report.generated_at.format(&self.options.timestamp_format)
        )
        .map_err(|e| FormatError::Io(e.to_string()))?;

        // Write sections
        for section in &report.sections {
            self.write_section(section, output)?;
        }

        // Write footer
        write!(
            output,
            r#"    <div class="footer">
        Report ID: {} | Generated by Tachikoma Analytics
    </div>
</body>
</html>"#,
            report.id
        )
        .map_err(|e| FormatError::Io(e.to_string()))?;

        Ok(())
    }
}

impl HtmlWriter {
    fn write_section(&self, section: &ReportSection, output: &mut dyn Write) -> Result<(), FormatError> {
        match section {
            ReportSection::Summary(summary) => {
                write!(
                    output,
                    r#"    <div class="section">
        <h2>{}</h2>
"#,
                    self.escape_html(&summary.title)
                )
                .map_err(|e| FormatError::Io(e.to_string()))?;

                for metric in &summary.metrics {
                    write!(
                        output,
                        r#"        <div class="metric">
            <div class="metric-value">{}</div>
            <div class="metric-name">{}</div>
        </div>
"#,
                        self.escape_html(&metric.value),
                        self.escape_html(&metric.name)
                    )
                    .map_err(|e| FormatError::Io(e.to_string()))?;
                }

                write!(output, "    </div>\n")
                    .map_err(|e| FormatError::Io(e.to_string()))?;
            }

            ReportSection::Table(table) => {
                write!(
                    output,
                    r#"    <div class="section">
        <h2>{}</h2>
        <table>
            <thead>
                <tr>
"#,
                    self.escape_html(&table.title)
                )
                .map_err(|e| FormatError::Io(e.to_string()))?;

                for header in &table.headers {
                    write!(output, "                    <th>{}</th>\n", self.escape_html(header))
                        .map_err(|e| FormatError::Io(e.to_string()))?;
                }

                write!(
                    output,
                    r#"                </tr>
            </thead>
            <tbody>
"#
                )
                .map_err(|e| FormatError::Io(e.to_string()))?;

                for row in &table.rows {
                    write!(output, "                <tr>\n")
                        .map_err(|e| FormatError::Io(e.to_string()))?;
                    for cell in row {
                        write!(output, "                    <td>{}</td>\n", self.escape_html(cell))
                            .map_err(|e| FormatError::Io(e.to_string()))?;
                    }
                    write!(output, "                </tr>\n")
                        .map_err(|e| FormatError::Io(e.to_string()))?;
                }

                write!(
                    output,
                    r#"            </tbody>
        </table>
    </div>
"#
                )
                .map_err(|e| FormatError::Io(e.to_string()))?;
            }

            _ => {
                // Handle other section types
            }
        }

        Ok(())
    }
}

/// Markdown report writer
pub struct MarkdownWriter {
    options: FormatOptions,
}

impl MarkdownWriter {
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }
}

impl FormatWriter for MarkdownWriter {
    fn write_events(&self, events: &[AnalyticsEvent], output: &mut dyn Write) -> Result<(), FormatError> {
        writeln!(output, "# Analytics Events Export\n")
            .map_err(|e| FormatError::Io(e.to_string()))?;
        writeln!(output, "Exported: {}\n", Utc::now().format(&self.options.timestamp_format))
            .map_err(|e| FormatError::Io(e.to_string()))?;

        writeln!(output, "| ID | Category | Event Type | Timestamp | Priority |")
            .map_err(|e| FormatError::Io(e.to_string()))?;
        writeln!(output, "|---|---|---|---|---|")
            .map_err(|e| FormatError::Io(e.to_string()))?;

        for event in events {
            writeln!(
                output,
                "| {:?} | {:?} | {:?} | {} | {:?} |",
                event.id,
                event.category,
                event.event_type,
                event.timestamp.format(&self.options.timestamp_format),
                event.priority
            )
            .map_err(|e| FormatError::Io(e.to_string()))?;
        }

        Ok(())
    }

    fn write_report(&self, report: &Report, output: &mut dyn Write) -> Result<(), FormatError> {
        writeln!(output, "# {}\n", report.title)
            .map_err(|e| FormatError::Io(e.to_string()))?;

        writeln!(
            output,
            "**Period:** {} to {}\n",
            report.period.start.format("%Y-%m-%d"),
            report.period.end.format("%Y-%m-%d")
        )
        .map_err(|e| FormatError::Io(e.to_string()))?;

        writeln!(
            output,
            "**Generated:** {}\n",
            report.generated_at.format(&self.options.timestamp_format)
        )
        .map_err(|e| FormatError::Io(e.to_string()))?;

        for section in &report.sections {
            self.write_section(section, output)?;
        }

        writeln!(output, "\n---\n*Report ID: {}*", report.id)
            .map_err(|e| FormatError::Io(e.to_string()))?;

        Ok(())
    }
}

impl MarkdownWriter {
    fn write_section(&self, section: &ReportSection, output: &mut dyn Write) -> Result<(), FormatError> {
        match section {
            ReportSection::Summary(summary) => {
                writeln!(output, "\n## {}\n", summary.title)
                    .map_err(|e| FormatError::Io(e.to_string()))?;

                for metric in &summary.metrics {
                    writeln!(output, "- **{}:** {}", metric.name, metric.value)
                        .map_err(|e| FormatError::Io(e.to_string()))?;
                }
            }

            ReportSection::Table(table) => {
                writeln!(output, "\n## {}\n", table.title)
                    .map_err(|e| FormatError::Io(e.to_string()))?;

                // Header
                writeln!(output, "| {} |", table.headers.join(" | "))
                    .map_err(|e| FormatError::Io(e.to_string()))?;

                // Separator
                let separator: Vec<&str> = table.headers.iter().map(|_| "---").collect();
                writeln!(output, "| {} |", separator.join(" | "))
                    .map_err(|e| FormatError::Io(e.to_string()))?;

                // Rows
                for row in &table.rows {
                    writeln!(output, "| {} |", row.join(" | "))
                        .map_err(|e| FormatError::Io(e.to_string()))?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}

/// Format factory
pub fn create_writer(format: ExportFormat, options: FormatOptions) -> Box<dyn FormatWriter> {
    match format {
        ExportFormat::Json => Box::new(JsonWriter::new(options)),
        ExportFormat::Csv => Box::new(CsvWriter::new(options)),
        ExportFormat::Ndjson => Box::new(NdjsonWriter::new(options)),
        ExportFormat::Html => Box::new(HtmlWriter::new(options)),
        ExportFormat::Markdown => Box::new(MarkdownWriter::new(options)),
        // For formats requiring external libraries
        ExportFormat::Parquet | ExportFormat::Pdf | ExportFormat::Excel => {
            // Fall back to JSON for unsupported formats
            Box::new(JsonWriter::new(options))
        }
    }
}

/// Format errors
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Unsupported format: {0}")]
    Unsupported(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::types::{EventBuilder, EventType};

    fn create_test_events() -> Vec<AnalyticsEvent> {
        vec![
            EventBuilder::new(EventType::SessionStarted).build(),
            EventBuilder::new(EventType::MissionCreated).build(),
            EventBuilder::new(EventType::FeatureUsed)
                .usage_data("test", "action", true)
                .build(),
        ]
    }

    #[test]
    fn test_json_writer() {
        let events = create_test_events();
        let writer = JsonWriter::new(FormatOptions::default());
        let mut output = Vec::new();

        writer.write_events(&events, &mut output).unwrap();

        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_csv_writer() {
        let events = create_test_events();
        let writer = CsvWriter::new(FormatOptions::default());
        let mut output = Vec::new();

        writer.write_events(&events, &mut output).unwrap();

        let csv = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = csv.lines().collect();

        // Header + 3 data rows
        assert_eq!(lines.len(), 4);
        assert!(lines[0].contains("id"));
        assert!(lines[0].contains("category"));
    }

    #[test]
    fn test_ndjson_writer() {
        let events = create_test_events();
        let writer = NdjsonWriter::new(FormatOptions::default());
        let mut output = Vec::new();

        writer.write_events(&events, &mut output).unwrap();

        let ndjson = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = ndjson.lines().collect();

        assert_eq!(lines.len(), 3);

        // Each line should be valid JSON
        for line in lines {
            let _: serde_json::Value = serde_json::from_str(line).unwrap();
        }
    }

    #[test]
    fn test_csv_escaping() {
        let writer = CsvWriter::new(FormatOptions::default());

        assert_eq!(writer.escape_csv_field("simple"), "simple");
        assert_eq!(writer.escape_csv_field("has,comma"), "\"has,comma\"");
        assert_eq!(writer.escape_csv_field("has\"quote"), "\"has\"\"quote\"");
    }

    #[test]
    fn test_format_factory() {
        let options = FormatOptions::default();

        let json_writer = create_writer(ExportFormat::Json, options.clone());
        let csv_writer = create_writer(ExportFormat::Csv, options.clone());
        let md_writer = create_writer(ExportFormat::Markdown, options);

        // Verify they can write events
        let events = create_test_events();
        let mut output = Vec::new();

        json_writer.write_events(&events, &mut output).unwrap();
        assert!(!output.is_empty());
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Each format writer
   - CSV escaping
   - HTML escaping
   - Format factory

2. **Integration Tests**
   - Full export pipeline
   - Large file exports
   - Report formatting

3. **Format Validation Tests**
   - JSON schema validity
   - CSV parsing
   - HTML structure

---

## Related Specs

- Spec 411: Analytics Export
- Spec 421: Report Generation
- Spec 424: Data Retention
