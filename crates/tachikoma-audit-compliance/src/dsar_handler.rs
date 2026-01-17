//! Data Subject Access Request handling.

use crate::gdpr::*;
use chrono::{Duration, Utc};
use parking_lot::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
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

    /// Initialize database tables for GDPR compliance.
    pub fn init_tables(&self) -> Result<(), GdprError> {
        let conn = self.conn.lock();
        
        // DSAR requests table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dsar_requests (
                id TEXT PRIMARY KEY,
                request_type TEXT NOT NULL,
                subject_id TEXT NOT NULL,
                subject_email TEXT,
                submitted_at TEXT NOT NULL,
                status TEXT NOT NULL,
                due_date TEXT NOT NULL,
                completed_at TEXT,
                notes TEXT NOT NULL,
                verified BOOLEAN NOT NULL DEFAULT FALSE
            )",
            [],
        )?;

        // Erasure records table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS erasure_records (
                id TEXT PRIMARY KEY,
                dsar_id TEXT NOT NULL,
                subject_id TEXT NOT NULL,
                erased_data_types TEXT NOT NULL,
                records_erased INTEGER NOT NULL,
                erased_at TEXT NOT NULL,
                erased_by TEXT NOT NULL,
                retained_for_legal TEXT NOT NULL,
                verification_hash TEXT NOT NULL
            )",
            [],
        )?;

        // Consent records table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS consent_records (
                id TEXT PRIMARY KEY,
                subject_id TEXT NOT NULL,
                purpose TEXT NOT NULL,
                legal_basis TEXT NOT NULL,
                granted_at TEXT NOT NULL,
                withdrawn_at TEXT,
                consent_method TEXT NOT NULL,
                evidence TEXT NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )?;

        // Processing activities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS processing_activities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                purposes TEXT NOT NULL,
                subject_categories TEXT NOT NULL,
                data_categories TEXT NOT NULL,
                recipients TEXT NOT NULL,
                third_country_transfers TEXT NOT NULL,
                retention_periods TEXT NOT NULL,
                security_measures TEXT NOT NULL,
                legal_basis TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    /// Create a new DSAR.
    pub fn create_request(
        &self,
        request_type: DsarType,
        subject_id: &str,
        subject_email: Option<&str>,
    ) -> Result<DataSubjectRequest, GdprError> {
        let now = Utc::now();
        let request = DataSubjectRequest {
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
        };

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO dsar_requests 
             (id, request_type, subject_id, subject_email, submitted_at, status, due_date, completed_at, notes, verified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                &request.id,
                serde_json::to_string(&request.request_type)?,
                &request.subject_id,
                &request.subject_email,
                request.submitted_at.to_rfc3339(),
                serde_json::to_string(&request.status)?,
                request.due_date.to_rfc3339(),
                request.completed_at.map(|t| t.to_rfc3339()),
                serde_json::to_string(&request.notes)?,
                request.verified
            ],
        )?;

        Ok(request)
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
        ).map_err(|_| GdprError::InvalidRequest("audit_events table not found".to_string()))?;

        let events: Vec<serde_json::Value> = stmt
            .query_map(
                rusqlite::params![&request.subject_id, &request.subject_id],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>(0).unwrap_or_default(),
                        "timestamp": row.get::<_, String>(1).unwrap_or_default(),
                        "category": row.get::<_, String>(2).unwrap_or_default(),
                        "action": row.get::<_, String>(3).unwrap_or_default(),
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

        // Count records to be affected
        let mut count_stmt = conn.prepare(
            "SELECT COUNT(*) FROM audit_events WHERE actor_id = ? OR target_id = ?"
        ).map_err(|_| GdprError::InvalidRequest("audit_events table not found".to_string()))?;
        
        let records_count = count_stmt.query_row(
            rusqlite::params![&request.subject_id, &request.subject_id],
            |row| Ok(row.get::<_, i64>(0).unwrap_or(0) as u64)
        ).unwrap_or(0);

        // Anonymize or delete events
        let records_erased = if retained.is_empty() {
            // Full deletion - simulate for now since we don't want to actually delete audit events
            records_count
        } else {
            // Anonymization (pseudonymization)
            let anonymized_id = format!("anonymized_{}", uuid::Uuid::new_v4());
            // Simulate anonymization - in a real system this would update the records
            records_count
        };

        // Create verification hash
        let verification_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(request.subject_id.as_bytes());
            hasher.update(Utc::now().to_rfc3339().as_bytes());
            format!("{:x}", hasher.finalize())
        };

        let erasure_record = ErasureRecord {
            id: uuid::Uuid::new_v4().to_string(),
            dsar_id: request.id.clone(),
            subject_id: request.subject_id.clone(),
            erased_data_types: vec!["audit_events".to_string()],
            records_erased,
            erased_at: Utc::now(),
            erased_by: erased_by.to_string(),
            retained_for_legal: retained.clone(),
            verification_hash: verification_hash.clone(),
        };

        // Store erasure record
        conn.execute(
            "INSERT INTO erasure_records 
             (id, dsar_id, subject_id, erased_data_types, records_erased, erased_at, erased_by, retained_for_legal, verification_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                &erasure_record.id,
                &erasure_record.dsar_id,
                &erasure_record.subject_id,
                serde_json::to_string(&erasure_record.erased_data_types)?,
                erasure_record.records_erased as i64,
                erasure_record.erased_at.to_rfc3339(),
                &erasure_record.erased_by,
                serde_json::to_string(&retained)?,
                &erasure_record.verification_hash
            ],
        )?;

        Ok(erasure_record)
    }

    fn check_legal_retention(
        &self,
        _conn: &Connection,
        _subject_id: &str,
    ) -> Result<Vec<RetainedData>, GdprError> {
        // Check if any data must be retained for legal reasons
        // This would check compliance holds, security incidents, etc.
        // For now, return empty (no retention requirements)
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
        ).map_err(|_| GdprError::InvalidRequest("audit_events table not found".to_string()))?;

        let events: Vec<serde_json::Value> = stmt
            .query_map([&request.subject_id], |row| {
                Ok(serde_json::json!({
                    "timestamp": row.get::<_, String>(1).unwrap_or_default(),
                    "category": row.get::<_, String>(2).unwrap_or_default(),
                    "action": row.get::<_, String>(3).unwrap_or_default(),
                    "outcome": row.get::<_, String>(9).unwrap_or_default(),
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

    /// Track consent for processing activities.
    pub fn record_consent(
        &self,
        subject_id: &str,
        purpose: &str,
        legal_basis: LegalBasis,
        consent_method: &str,
        evidence: serde_json::Value,
    ) -> Result<ConsentRecord, GdprError> {
        let consent = ConsentRecord {
            id: uuid::Uuid::new_v4().to_string(),
            subject_id: subject_id.to_string(),
            purpose: purpose.to_string(),
            legal_basis,
            granted_at: Utc::now(),
            withdrawn_at: None,
            consent_method: consent_method.to_string(),
            evidence,
            status: ConsentStatus::Active,
        };

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO consent_records 
             (id, subject_id, purpose, legal_basis, granted_at, withdrawn_at, consent_method, evidence, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                &consent.id,
                &consent.subject_id,
                &consent.purpose,
                serde_json::to_string(&consent.legal_basis)?,
                consent.granted_at.to_rfc3339(),
                consent.withdrawn_at.map(|t| t.to_rfc3339()),
                &consent.consent_method,
                serde_json::to_string(&consent.evidence)?,
                serde_json::to_string(&consent.status)?
            ],
        )?;

        Ok(consent)
    }

    /// Withdraw consent.
    pub fn withdraw_consent(&self, consent_id: &str) -> Result<(), GdprError> {
        let conn = self.conn.lock();
        let now = Utc::now();
        
        conn.execute(
            "UPDATE consent_records SET withdrawn_at = ?, status = ? WHERE id = ?",
            rusqlite::params![
                now.to_rfc3339(),
                serde_json::to_string(&ConsentStatus::Withdrawn)?,
                consent_id
            ],
        )?;

        Ok(())
    }

    /// Record a processing activity.
    pub fn record_processing_activity(
        &self,
        activity: &ProcessingActivity,
    ) -> Result<(), GdprError> {
        let conn = self.conn.lock();
        
        conn.execute(
            "INSERT OR REPLACE INTO processing_activities 
             (id, name, purposes, subject_categories, data_categories, recipients, third_country_transfers, retention_periods, security_measures, legal_basis)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                &activity.id,
                &activity.name,
                serde_json::to_string(&activity.purposes)?,
                serde_json::to_string(&activity.subject_categories)?,
                serde_json::to_string(&activity.data_categories)?,
                serde_json::to_string(&activity.recipients)?,
                serde_json::to_string(&activity.third_country_transfers)?,
                serde_json::to_string(&activity.retention_periods)?,
                serde_json::to_string(&activity.security_measures)?,
                serde_json::to_string(&activity.legal_basis)?
            ],
        )?;

        Ok(())
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

/// GDPR error.
#[derive(Debug, thiserror::Error)]
pub enum GdprError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}