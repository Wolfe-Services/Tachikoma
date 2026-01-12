//! Timestamp utilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A UTC timestamp.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Current time.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// From a DateTime.
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Get the inner DateTime.
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// ISO 8601 string.
    pub fn to_iso8601(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Duration since this timestamp.
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now() - self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timestamp({})", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_serialization() {
        let ts = Timestamp::now();
        let json = serde_json::to_string(&ts).unwrap();
        let deserialized: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(ts, deserialized);
    }

    #[test]
    fn test_timestamp_display() {
        let ts = Timestamp::now();
        let display_str = ts.to_string();
        assert!(display_str.contains('T')); // ISO 8601 format should contain T
        // UTC timestamp should end with Z or +00:00
        assert!(display_str.ends_with('Z') || display_str.ends_with("+00:00"));
    }

    #[test]
    fn test_timestamp_ordering() {
        let ts1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let ts2 = Timestamp::now();
        assert!(ts1 < ts2);
    }
}