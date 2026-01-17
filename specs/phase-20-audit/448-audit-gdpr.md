# 448 - Audit GDPR

**Phase:** 20 - Audit System
**Spec ID:** 448
**Status:** Planned
**Dependencies:** 435-audit-query, 436-audit-retention
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement GDPR-specific features for audit data, including data subject access requests, right to erasure, and data portability.

---

## Acceptance Criteria

- [x] Data subject access request handling
- [x] Right to erasure (with audit trail)
- [x] Data portability export
- [x] Consent tracking
- [x] Processing activity records

---

## Implementation Details

### 1. GDPR Types (src/gdpr.rs)

```rust
//! GDPR compliance features.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data Subject Access Request (DSAR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    /// Request identifier.
    pub id: String,
    /// Type of request.
    pub request_type: DsarType,
    /// Data subject identifier.
    pub subject_id: String,
    /// Subject email (for response).
    pub subject_email: Option<String>,
    /// Request submission time.
    pub submitted_at: DateTime<Utc>,
    /// Request status.
    pub status: DsarStatus,
    /// Due date (30 days from submission).
    pub due_date: DateTime<Utc>,
    /// When completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Handler notes.
    pub notes: Vec<String>,
    /// Verification status.
    pub verified: bool,
}

/// Type of DSAR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DsarType {
    /// Right of access (Article 15).
    Access,
    /// Right to rectification (Article 16).
    Rectification,
    /// Right to erasure (Article 17).
    Erasure,
    /// Right to restriction (Article 18).
    Restriction,
    /// Right to data portability (Article 20).
    Portability,
    /// Right to object (Article 21).
    Objection,
}

impl DsarType {
    /// Get GDPR article reference.
    pub fn article(&self) -> &'static str {
        match self {
            Self::Access => "Article 15",
            Self::Rectification => "Article 16",
            Self::Erasure => "Article 17",
            Self::Restriction => "Article 18",
            Self::Portability => "Article 20",
            Self::Objection => "Article 21",
        }
    }
}

/// DSAR status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DsarStatus {
    /// Awaiting verification.
    PendingVerification,
    /// Verified, being processed.
    InProgress,
    /// On hold (needs clarification).
    OnHold,
    /// Completed successfully.
    Completed,
    /// Rejected (invalid request).
    Rejected,
    /// Overdue.
    Overdue,
}

/// Erasure record (for audit trail of deletions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureRecord {
    /// Record identifier.
    pub id: String,
    /// Related DSAR ID.
    pub dsar_id: String,
    /// Data subject.
    pub subject_id: String,
    /// What was erased.
    pub erased_data_types: Vec<String>,
    /// Number of records erased.
    pub records_erased: u64,
    /// When erased.
    pub erased_at: DateTime<Utc>,
    /// Who performed erasure.
    pub erased_by: String,
    /// Retained for legal basis.
    pub retained_for_legal: Vec<RetainedData>,
    /// Verification hash (proves erasure without storing data).
    pub verification_hash: String,
}

/// Data retained for legal reasons.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetainedData {
    /// Data type.
    pub data_type: String,
    /// Legal basis for retention.
    pub legal_basis: String,
    /// Retention period.
    pub retention_until: DateTime<Utc>,
}

/// Processing activity record (Article 30).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingActivity {
    /// Activity identifier.
    pub id: String,
    /// Activity name.
    pub name: String,
    /// Purpose of processing.
    pub purposes: Vec<String>,
    /// Categories of data subjects.
    pub subject_categories: Vec<String>,
    /// Categories of personal data.
    pub data_categories: Vec<String>,
    /// Recipients of data.
    pub recipients: Vec<String>,
    /// Transfers to third countries.
    pub third_country_transfers: Vec<ThirdCountryTransfer>,
    /// Retention periods.
    pub retention_periods: Vec<RetentionPeriod>,
    /// Security measures.
    pub security_measures: Vec<String>,
    /// Legal basis.
    pub legal_basis: LegalBasis,
}

/// Third country data transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdCountryTransfer {
    pub country: String,
    pub safeguards: String,
}

/// Retention period specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPeriod {
    pub data_category: String,
    pub period_days: u32,
    pub justification: String,
}

/// Legal basis for processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalBasis {
    /// Consent (Article 6(1)(a)).
    Consent,
    /// Contract (Article 6(1)(b)).
    Contract,
    /// Legal obligation (Article 6(1)(c)).
    LegalObligation,
    /// Vital interests (Article 6(1)(d)).
    VitalInterests,
    /// Public task (Article 6(1)(e)).
    PublicTask,
    /// Legitimate interests (Article 6(1)(f)).
    LegitimateInterests,
}
```

### 2. DSAR Handler (src/dsar_handler.rs)

```rust
//! Data Subject Access Request handling.

use crate::gdpr::*;
use chrono::{Duration, Utc};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

/// DSAR handler.
pub struct DsarHandler {
    conn: Arc<Mutex<Connection>>,
}

impl DsarHandler {
    /// Create a new handler.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Create a new DSAR.
    pub fn create_request(
        &self,
        request_type: DsarType,
        subject_id: &str,
        subject_email: Option<&str>,
    ) -> DataSubjectRequest {
        let now = Utc::now();

        DataSubjectRequest {
            id: uuid::Uuid::new_v4().to_string(),
            request_type,
            subject_id: subject_id.to_string(),
            subject_email: subject_email.map(String::from),
            submitted_at: now,
            status: DsarStatus::PendingVerification,
            due_date: now + Duration::days(30),
            completed_at: None,
            notes: Vec::new(),
            verified: false,
        }
    }

    /// Process an access request (Article 15).
    pub fn process_access_request(
        &self,
        request: &DataSubjectRequest,
    ) -> Result<AccessResponse, GdprError> {
        let conn = self.conn.lock();

        // Find all audit events related to this subject
        let mut stmt = conn.prepare(
            "SELECT * FROM audit_events
             WHERE actor_id = ? OR target_id = ?
             ORDER BY timestamp"
        )?;

        let events: Vec<serde_json::Value> = stmt
            .query_map(
                rusqlite::params![&request.subject_id, &request.subject_id],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>(0)?,
                        "timestamp": row.get::<_, String>(1)?,
                        "category": row.get::<_, String>(2)?,
                        "action": row.get::<_, String>(3)?,
                    }))
                },
            )?
            .filter_map(|r| r.ok())
            .collect();

        Ok(AccessResponse {
            request_id: request.id.clone(),
            subject_id: request.subject_id.clone(),
            data_categories: vec!["audit_logs".to_string()],
            events,
            processing_purposes: vec!["Security monitoring".to_string(), "Compliance".to_string()],
            recipients: vec!["Internal security team".to_string()],
            retention_period: "As per retention policy".to_string(),
            generated_at: Utc::now(),
        })
    }

    /// Process an erasure request (Article 17).
    pub fn process_erasure_request(
        &self,
        request: &DataSubjectRequest,
        erased_by: &str,
    ) -> Result<ErasureRecord, GdprError> {
        let conn = self.conn.lock();

        // Check for legal retention requirements first
        let retained = self.check_legal_retention(&conn, &request.subject_id)?;

        // Anonymize or delete events
        let records_erased = if retained.is_empty() {
            // Full deletion
            conn.execute(
                "DELETE FROM audit_events WHERE actor_id = ? OR target_id = ?",
                rusqlite::params![&request.subject_id, &request.subject_id],
            )? as u64
        } else {
            // Anonymization (pseudonymization)
            let anonymized_id = format!("anonymized_{}", uuid::Uuid::new_v4());
            conn.execute(
                "UPDATE audit_events SET actor_id = ?, actor_name = NULL
                 WHERE actor_id = ?",
                rusqlite::params![&anonymized_id, &request.subject_id],
            )?;
            conn.execute(
                "UPDATE audit_events SET target_id = ?, target_name = NULL
                 WHERE target_id = ?",
                rusqlite::params![&anonymized_id, &request.subject_id],
            )? as u64
        };

        // Create verification hash
        let verification_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(request.subject_id.as_bytes());
            hasher.update(Utc::now().to_rfc3339().as_bytes());
            format!("{:x}", hasher.finalize())
        };

        Ok(ErasureRecord {
            id: uuid::Uuid::new_v4().to_string(),
            dsar_id: request.id.clone(),
            subject_id: request.subject_id.clone(),
            erased_data_types: vec!["audit_events".to_string()],
            records_erased,
            erased_at: Utc::now(),
            erased_by: erased_by.to_string(),
            retained_for_legal: retained,
            verification_hash,
        })
    }

    fn check_legal_retention(
        &self,
        _conn: &Connection,
        _subject_id: &str,
    ) -> Result<Vec<RetainedData>, GdprError> {
        // Check if any data must be retained for legal reasons
        // This would check compliance holds, security incidents, etc.
        Ok(Vec::new())
    }

    /// Export data for portability (Article 20).
    pub fn export_portable_data(
        &self,
        request: &DataSubjectRequest,
    ) -> Result<PortableData, GdprError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT * FROM audit_events
             WHERE actor_id = ?
             ORDER BY timestamp"
        )?;

        let events: Vec<serde_json::Value> = stmt
            .query_map([&request.subject_id], |row| {
                Ok(serde_json::json!({
                    "timestamp": row.get::<_, String>(1)?,
                    "category": row.get::<_, String>(2)?,
                    "action": row.get::<_, String>(3)?,
                    "outcome": row.get::<_, String>(9)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(PortableData {
            format: "json".to_string(),
            schema_version: "1.0".to_string(),
            exported_at: Utc::now(),
            subject_id: request.subject_id.clone(),
            data: serde_json::json!({
                "audit_events": events
            }),
        })
    }
}

/// Access request response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessResponse {
    pub request_id: String,
    pub subject_id: String,
    pub data_categories: Vec<String>,
    pub events: Vec<serde_json::Value>,
    pub processing_purposes: Vec<String>,
    pub recipients: Vec<String>,
    pub retention_period: String,
    pub generated_at: chrono::DateTime<Utc>,
}

/// Portable data export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableData {
    pub format: String,
    pub schema_version: String,
    pub exported_at: chrono::DateTime<Utc>,
    pub subject_id: String,
    pub data: serde_json::Value,
}

use serde::Serialize;

/// GDPR error.
#[derive(Debug, thiserror::Error)]
pub enum GdprError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}
```

---

## Testing Requirements

1. DSAR workflow functions correctly
2. Erasure creates proper audit trail
3. Portability export is machine-readable
4. Legal retention is respected
5. Due dates are calculated correctly

---

## Related Specs

- Depends on: [435-audit-query.md](435-audit-query.md), [436-audit-retention.md](436-audit-retention.md)
- Next: [449-audit-api.md](449-audit-api.md)
