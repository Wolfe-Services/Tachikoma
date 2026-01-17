//! Audit export functionality for Tachikoma.

mod export;
mod json_export;
mod csv_export;
mod siem_export;

pub use export::{
    ExportConfig, ExportError, ExportFormat, ExportProgress, ExportResult, ExportWriter,
    ProgressCallback,
};
pub use json_export::{JsonLinesExporter, JsonPrettyExporter};
pub use csv_export::CsvExporter;
pub use siem_export::{CefExporter, LeefExporter};

/// Create an export writer for the given format and configuration.
pub fn create_exporter<W: std::io::Write + Send + 'static>(
    writer: W,
    config: ExportConfig,
) -> ExportResult<Box<dyn ExportWriter>> {
    let boxed: Box<dyn ExportWriter> = match config.format {
        ExportFormat::JsonLines => Box::new(JsonLinesExporter::new(writer, config)),
        ExportFormat::JsonPretty => Box::new(JsonPrettyExporter::new(writer, config)),
        ExportFormat::Csv => Box::new(CsvExporter::new(writer, config)),
        ExportFormat::Cef => Box::new(CefExporter::new(writer, config)),
        ExportFormat::Leef => Box::new(LeefExporter::new(writer, config)),
    };
    Ok(boxed)
}