//! Snapshot testing utilities using insta.
//!
//! Snapshot tests capture complex output and compare against stored baselines.
//! Use `cargo insta review` to interactively accept/reject changes.

use insta::{assert_json_snapshot, assert_yaml_snapshot, assert_debug_snapshot};
use serde::Serialize;

/// Configuration for snapshot behavior
pub struct SnapshotConfig {
    /// Snapshot directory relative to crate root
    pub snapshot_dir: &'static str,
    /// Whether to sort keys in JSON/YAML output
    pub sort_keys: bool,
    /// Redactions to apply (hide dynamic values)
    pub redactions: Vec<(&'static str, &'static str)>,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            snapshot_dir: "snapshots",
            sort_keys: true,
            redactions: vec![
                ("[].id", "[id]"),
                ("[].created_at", "[timestamp]"),
                ("[].updated_at", "[timestamp]"),
            ],
        }
    }
}

/// Helper to create consistent snapshot settings
#[macro_export]
macro_rules! snapshot_settings {
    () => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path("snapshots");
        settings.set_prepend_module_to_snapshot(false);
        settings.set_sort_maps(true);
        settings
    }};
}

/// Assert a JSON snapshot with standard settings
#[macro_export]
macro_rules! assert_json {
    ($value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_json_snapshot!($value);
        });
    }};
    ($name:expr, $value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_json_snapshot!($name, $value);
        });
    }};
}

/// Assert a YAML snapshot with standard settings
#[macro_export]
macro_rules! assert_yaml {
    ($value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_yaml_snapshot!($value);
        });
    }};
    ($name:expr, $value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_yaml_snapshot!($name, $value);
        });
    }};
}

/// Assert a debug snapshot with standard settings
#[macro_export]
macro_rules! assert_debug {
    ($value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_debug_snapshot!($value);
        });
    }};
    ($name:expr, $value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_debug_snapshot!($name, $value);
        });
    }};
}

/// Redact dynamic fields in snapshots
pub fn with_redactions<F, R>(redactions: &[(&str, &str)], f: F) -> R
where
    F: FnOnce() -> R,
{
    let mut settings = insta::Settings::clone_current();
    for (selector, placeholder) in redactions {
        settings.add_redaction(selector, placeholder);
    }
    settings.bind(f)
}