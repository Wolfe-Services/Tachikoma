//! Audit event severity levels.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Severity level for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational events (normal operations).
    Info,
    /// Low-impact events that may warrant review.
    Low,
    /// Medium-impact events requiring attention.
    Medium,
    /// High-impact events requiring immediate review.
    High,
    /// Critical security events.
    Critical,
}

impl AuditSeverity {
    /// Numeric value for comparison (higher = more severe).
    pub fn level(&self) -> u8 {
        match self {
            Self::Info => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    /// Check if this severity meets a minimum threshold.
    pub fn meets_threshold(&self, threshold: Self) -> bool {
        self.level() >= threshold.level()
    }
}

impl PartialOrd for AuditSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AuditSeverity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level().cmp(&other.level())
    }
}

impl Default for AuditSeverity {
    fn default() -> Self {
        Self::Info
    }
}