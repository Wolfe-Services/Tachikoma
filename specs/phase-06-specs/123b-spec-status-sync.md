# 123b - Spec Status Auto-Sync

**Phase:** 6 - Spec System
**Spec ID:** 123b
**Status:** Planned
**Dependencies:** 123-checkbox-tracking
**Estimated Context:** ~3% of Sonnet window

---

## Objective

Automatically update spec status based on checkbox completion. A spec with all checkboxes checked should be "Complete", not "Planned". This prevents metadata/content mismatch.

---

## Problem

Current specs have inconsistent status:
- Checkboxes all checked but `Status: Planned`
- Breaks progress tracking and reporting
- Confuses both humans and agents

---

## Acceptance Criteria

- [ ] Detect checkbox completion percentage
- [ ] Auto-update status based on rules
- [ ] Pre-commit hook for validation
- [ ] CLI command to sync all specs
- [ ] Warning on status/checkbox mismatch

---

## Implementation Details

### Status Rules

```
0% complete    → Status: Planned
1-99% complete → Status: In Progress
100% complete  → Status: Complete
```

### src/spec/status_sync.rs

```rust
//! Spec status auto-sync based on checkbox completion.

use std::path::Path;
use regex::Regex;

/// Spec status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecStatus {
    Planned,
    InProgress,
    Complete,
    Blocked,
}

impl SpecStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "planned" => Some(Self::Planned),
            "in progress" | "in-progress" => Some(Self::InProgress),
            "complete" | "completed" => Some(Self::Complete),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::InProgress => "In Progress",
            Self::Complete => "Complete",
            Self::Blocked => "Blocked",
        }
    }
}

/// Checkbox state in a spec.
#[derive(Debug, Default)]
pub struct CheckboxState {
    pub total: usize,
    pub checked: usize,
}

impl CheckboxState {
    pub fn completion_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.checked as f64 / self.total as f64) * 100.0
        }
    }

    pub fn expected_status(&self) -> SpecStatus {
        let percent = self.completion_percent();
        if percent == 0.0 {
            SpecStatus::Planned
        } else if percent >= 100.0 {
            SpecStatus::Complete
        } else {
            SpecStatus::InProgress
        }
    }
}

/// Parse checkbox state from spec content.
pub fn parse_checkboxes(content: &str) -> CheckboxState {
    let checked_re = Regex::new(r"^\s*-\s*\[x\]").unwrap();
    let unchecked_re = Regex::new(r"^\s*-\s*\[\s?\]").unwrap();

    let mut state = CheckboxState::default();

    for line in content.lines() {
        if checked_re.is_match(line) {
            state.total += 1;
            state.checked += 1;
        } else if unchecked_re.is_match(line) {
            state.total += 1;
        }
    }

    state
}

/// Parse current status from spec content.
pub fn parse_status(content: &str) -> Option<SpecStatus> {
    let status_re = Regex::new(r"(?i)\*\*Status:\*\*\s*(\w+(?:\s+\w+)?)").unwrap();

    status_re
        .captures(content)
        .and_then(|caps| caps.get(1))
        .and_then(|m| SpecStatus::from_str(m.as_str()))
}

/// Check if status matches checkbox state.
pub fn status_matches_checkboxes(content: &str) -> StatusMatch {
    let checkboxes = parse_checkboxes(content);
    let current_status = parse_status(content);
    let expected_status = checkboxes.expected_status();

    match current_status {
        Some(status) if status == expected_status => StatusMatch::Ok,
        Some(status) => StatusMatch::Mismatch {
            current: status,
            expected: expected_status,
            completion: checkboxes.completion_percent(),
        },
        None => StatusMatch::Missing,
    }
}

/// Result of status check.
#[derive(Debug)]
pub enum StatusMatch {
    Ok,
    Mismatch {
        current: SpecStatus,
        expected: SpecStatus,
        completion: f64,
    },
    Missing,
}

/// Update status in spec content.
pub fn update_status(content: &str, new_status: SpecStatus) -> String {
    let status_re = Regex::new(r"(?i)(\*\*Status:\*\*\s*)(\w+(?:\s+\w+)?)").unwrap();

    status_re
        .replace(content, format!("${{1}}{}", new_status.as_str()))
        .to_string()
}

/// Sync status for a spec file.
pub fn sync_spec_status(path: &Path) -> Result<SyncResult, std::io::Error> {
    let content = std::fs::read_to_string(path)?;

    match status_matches_checkboxes(&content) {
        StatusMatch::Ok => Ok(SyncResult::AlreadySynced),
        StatusMatch::Mismatch { expected, .. } => {
            let updated = update_status(&content, expected);
            std::fs::write(path, &updated)?;
            Ok(SyncResult::Updated(expected))
        }
        StatusMatch::Missing => Ok(SyncResult::NoStatus),
    }
}

/// Result of sync operation.
#[derive(Debug)]
pub enum SyncResult {
    AlreadySynced,
    Updated(SpecStatus),
    NoStatus,
}
```

### CLI Command

```bash
# Sync all specs
tachikoma spec sync-status

# Check without modifying
tachikoma spec check-status

# Sync single spec
tachikoma spec sync-status specs/phase-XX/NNN-spec.md
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit (or .jj/hooks/)

# Check for status/checkbox mismatches
mismatches=$(tachikoma spec check-status --json | jq '.mismatches | length')

if [ "$mismatches" -gt 0 ]; then
    echo "ERROR: Spec status/checkbox mismatches detected"
    tachikoma spec check-status
    echo ""
    echo "Run 'tachikoma spec sync-status' to fix"
    exit 1
fi
```

---

## Testing Requirements

1. Parse checkboxes correctly (checked and unchecked)
2. Calculate completion percentage
3. Detect status mismatch
4. Update status correctly
5. Pre-commit hook blocks on mismatch

---

## Related Specs

- Depends on: [123-checkbox-tracking.md](123-checkbox-tracking.md)
- Related: [127-spec-validation.md](127-spec-validation.md)
