//! Spec linting functionality.

use crate::metadata::SpecMetadata;
use thiserror::Error;

/// Confidence level for lint rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Low confidence lint
    Low,
    /// Medium confidence lint
    Medium,
    /// High confidence lint
    High,
}

/// Lint error
#[derive(Debug, Error)]
pub enum LintError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Lint a spec
pub fn lint_spec(_metadata: &SpecMetadata) -> Result<Vec<String>, LintError> {
    // Stub implementation
    Ok(vec![])
}