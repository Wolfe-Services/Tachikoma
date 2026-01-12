//! Output format definitions and traits.

use serde::Serialize;

/// Internal output format (distinct from CLI's OutputFormat)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Table,
}

/// Trait for types that can be displayed in multiple formats
pub trait Displayable: Serialize + std::fmt::Display {
    /// Format as human-readable text (default: use Display impl)
    fn format_text(&self) -> String {
        self.to_string()
    }

    /// Format as JSON
    fn format_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Format as table (optional, return None if not supported)
    fn as_table(&self) -> Option<crate::table::Table> {
        None
    }
}

/// Blanket implementation for types that implement the required traits
impl<T> Displayable for T
where
    T: Serialize + std::fmt::Display,
{
}