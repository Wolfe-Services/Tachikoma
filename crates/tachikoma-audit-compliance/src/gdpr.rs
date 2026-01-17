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

/// Consent record for GDPR tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Consent identifier.
    pub id: String,
    /// Data subject.
    pub subject_id: String,
    /// What consent was given for.
    pub purpose: String,
    /// Legal basis category.
    pub legal_basis: LegalBasis,
    /// When consent was given.
    pub granted_at: DateTime<Utc>,
    /// When consent was withdrawn (if applicable).
    pub withdrawn_at: Option<DateTime<Utc>>,
    /// Method of consent.
    pub consent_method: String,
    /// Evidence of consent (e.g., form data, IP address).
    pub evidence: serde_json::Value,
    /// Current status.
    pub status: ConsentStatus,
}

/// Status of consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    /// Consent is active.
    Active,
    /// Consent has been withdrawn.
    Withdrawn,
    /// Consent has expired.
    Expired,
}