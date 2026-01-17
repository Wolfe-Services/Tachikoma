//! Flaky test detection and tracking.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Record of a flaky test occurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestRecord {
    pub test_name: String,
    pub module: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub failure_messages: Vec<String>,
    pub status: FlakyTestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FlakyTestStatus {
    Active,
    Quarantined,
    Fixed,
    Investigating,
}

impl FlakyTestRecord {
    pub fn new(test_name: String, module: String) -> Self {
        let now = Utc::now();
        Self {
            test_name,
            module,
            first_seen: now,
            last_seen: now,
            occurrence_count: 1,
            pass_count: 0,
            fail_count: 0,
            failure_messages: Vec::new(),
            status: FlakyTestStatus::Active,
        }
    }

    pub fn record_pass(&mut self) {
        self.pass_count += 1;
        self.occurrence_count += 1;
        self.last_seen = Utc::now();
    }

    pub fn record_fail(&mut self, message: Option<String>) {
        self.fail_count += 1;
        self.occurrence_count += 1;
        self.last_seen = Utc::now();
        if let Some(msg) = message {
            if self.failure_messages.len() < 10 {
                self.failure_messages.push(msg);
            }
        }
    }

    pub fn flaky_rate(&self) -> f64 {
        if self.occurrence_count == 0 {
            return 0.0;
        }
        let flaky = self.occurrence_count as f64 - self.pass_count.max(self.fail_count) as f64;
        flaky / self.occurrence_count as f64
    }

    pub fn is_flaky(&self) -> bool {
        self.occurrence_count >= 3 && self.pass_count > 0 && self.fail_count > 0
    }
}

/// Flaky test database
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FlakyTestDb {
    pub tests: HashMap<String, FlakyTestRecord>,
    pub last_updated: Option<DateTime<Utc>>,
}

impl FlakyTestDb {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    pub fn record_test_result(&mut self, test_name: &str, module: &str, passed: bool, message: Option<String>) {
        let key = format!("{}::{}", module, test_name);

        let record = self.tests
            .entry(key)
            .or_insert_with(|| FlakyTestRecord::new(test_name.to_string(), module.to_string()));

        if passed {
            record.record_pass();
        } else {
            record.record_fail(message);
        }

        self.last_updated = Some(Utc::now());
    }

    pub fn get_flaky_tests(&self) -> Vec<&FlakyTestRecord> {
        self.tests.values().filter(|t| t.is_flaky()).collect()
    }

    pub fn get_quarantined(&self) -> Vec<&FlakyTestRecord> {
        self.tests.values()
            .filter(|t| t.status == FlakyTestStatus::Quarantined)
            .collect()
    }

    pub fn quarantine(&mut self, test_key: &str) {
        if let Some(record) = self.tests.get_mut(test_key) {
            record.status = FlakyTestStatus::Quarantined;
        }
    }

    pub fn mark_fixed(&mut self, test_key: &str) {
        if let Some(record) = self.tests.get_mut(test_key) {
            record.status = FlakyTestStatus::Fixed;
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Flaky Test Report\n\n");

        let flaky = self.get_flaky_tests();
        if flaky.is_empty() {
            report.push_str("No flaky tests detected.\n");
            return report;
        }

        report.push_str(&format!("## Summary\n\nTotal flaky tests: {}\n\n", flaky.len()));

        report.push_str("## Active Flaky Tests\n\n");
        report.push_str("| Test | Module | Flaky Rate | Pass/Fail | Status |\n");
        report.push_str("|------|--------|------------|-----------|--------|\n");

        for test in &flaky {
            report.push_str(&format!(
                "| {} | {} | {:.1}% | {}/{} | {:?} |\n",
                test.test_name,
                test.module,
                test.flaky_rate() * 100.0,
                test.pass_count,
                test.fail_count,
                test.status
            ));
        }

        report
    }
}

/// Macro to mark a test as potentially flaky
#[macro_export]
macro_rules! flaky_test {
    ($name:ident, $body:expr) => {
        #[test]
        #[allow(non_snake_case)]
        fn $name() {
            // Tag for nextest filtering
            let _flaky_marker = "flaky";
            $body
        }
    };
}

/// Macro to skip quarantined tests
#[macro_export]
macro_rules! quarantined_test {
    ($name:ident, $reason:expr, $body:expr) => {
        #[test]
        #[ignore = $reason]
        fn $name() {
            $body
        }
    };
}