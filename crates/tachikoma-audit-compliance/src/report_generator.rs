//! Compliance report generation.

use crate::compliance::*;
use crate::control_library::ControlLibrary;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

/// Report generator configuration.
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Minimum events required for compliant status.
    pub min_evidence_count: u64,
    /// Include event samples in evidence.
    pub include_samples: bool,
    /// Maximum samples per control.
    pub max_samples: u32,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            min_evidence_count: 10,
            include_samples: true,
            max_samples: 5,
        }
    }
}

/// Compliance report generator.
pub struct ReportGenerator {
    conn: Arc<Mutex<Connection>>,
    library: ControlLibrary,
    config: ReportConfig,
}

impl ReportGenerator {
    /// Create a new report generator.
    pub fn new(
        conn: Arc<Mutex<Connection>>,
        library: ControlLibrary,
        config: ReportConfig,
    ) -> Self {
        Self { conn, library, config }
    }

    /// Generate a compliance report.
    pub fn generate(
        &self,
        framework: ComplianceFramework,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        generated_by: &str,
    ) -> Result<ComplianceReport, ReportError> {
        let controls = self.library.by_framework(framework);
        let mut assessments = Vec::new();

        for control in controls {
            let assessment = self.assess_control(control, period_start, period_end)?;
            assessments.push(assessment);
        }

        let summary = ComplianceSummary::from_assessments(&assessments);

        Ok(ComplianceReport {
            id: uuid::Uuid::new_v4().to_string(),
            framework,
            title: format!("{} Compliance Report", framework.display_name()),
            period_start,
            period_end,
            generated_at: Utc::now(),
            generated_by: generated_by.to_string(),
            controls: assessments,
            summary,
        })
    }

    fn assess_control(
        &self,
        control: &ComplianceControl,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<ControlAssessment, ReportError> {
        let conn = self.conn.lock();

        // Build query for evidence
        let categories: Vec<String> = control.evidence_categories
            .iter()
            .map(|c| c.to_string())
            .collect();

        let placeholders = categories.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id, timestamp, category, action FROM audit_events
             WHERE category IN ({}) AND timestamp >= ? AND timestamp < ?
             ORDER BY timestamp DESC",
            placeholders
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = categories
            .iter()
            .map(|c| Box::new(c.clone()) as Box<dyn rusqlite::ToSql>)
            .collect();
        params.push(Box::new(period_start.to_rfc3339()));
        params.push(Box::new(period_end.to_rfc3339()));

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let mut event_ids = Vec::new();
        let mut event_count = 0u64;

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(3)?))
        })?;

        for row in rows {
            let (id, action) = row?;
            event_count += 1;

            // Check if action matches required evidence
            let action_lower = action.to_lowercase();
            if control.evidence_actions.iter().any(|a| action_lower.contains(&a.to_lowercase())) {
                if self.config.include_samples && event_ids.len() < self.config.max_samples as usize {
                    event_ids.push(id);
                }
            }
        }

        // Determine status
        let status = if event_count >= self.config.min_evidence_count {
            ControlStatus::Compliant
        } else if event_count > 0 {
            ControlStatus::PartiallyCompliant
        } else {
            ControlStatus::Indeterminate
        };

        let evidence = vec![ComplianceEvidence {
            control_id: control.id.clone(),
            evidence_type: EvidenceType::AuditLog,
            description: format!("{} audit events in period", event_count),
            event_ids,
            period_start,
            period_end,
            event_count,
        }];

        let mut findings = Vec::new();
        let mut recommendations = Vec::new();

        if event_count < self.config.min_evidence_count {
            findings.push(format!(
                "Insufficient audit evidence ({} events, {} required)",
                event_count, self.config.min_evidence_count
            ));
            recommendations.push("Increase audit logging for this control area".to_string());
        }

        Ok(ControlAssessment {
            control: control.clone(),
            status,
            evidence,
            findings,
            recommendations,
        })
    }
}

/// Report generation error.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("configuration error: {0}")]
    Config(String),
}