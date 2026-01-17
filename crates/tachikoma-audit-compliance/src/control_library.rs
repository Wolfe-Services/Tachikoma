//! Pre-defined compliance controls.

use crate::compliance::*;
use tachikoma_audit_types::AuditCategory;
use std::collections::HashMap;

/// Library of compliance controls.
pub struct ControlLibrary {
    controls: HashMap<String, ComplianceControl>,
}

impl ControlLibrary {
    /// Create a new library with default controls.
    pub fn new() -> Self {
        let mut library = Self {
            controls: HashMap::new(),
        };
        library.load_soc2_controls();
        library.load_gdpr_controls();
        library
    }

    fn load_soc2_controls(&mut self) {
        // SOC 2 Common Criteria controls
        let controls = vec![
            ComplianceControl {
                id: "CC6.1".to_string(),
                name: "Logical Access Security".to_string(),
                description: "The entity implements logical access security software, infrastructure, and architectures over protected information assets".to_string(),
                framework: ComplianceFramework::Soc2,
                category: "Logical and Physical Access Controls".to_string(),
                evidence_categories: vec![AuditCategory::Authentication, AuditCategory::Authorization],
                evidence_actions: vec!["login".to_string(), "logout".to_string(), "access_granted".to_string(), "access_denied".to_string()],
            },
            ComplianceControl {
                id: "CC6.2".to_string(),
                name: "Access Provisioning".to_string(),
                description: "Prior to issuing system credentials, the entity registers and authorizes new internal and external users".to_string(),
                framework: ComplianceFramework::Soc2,
                category: "Logical and Physical Access Controls".to_string(),
                evidence_categories: vec![AuditCategory::UserManagement],
                evidence_actions: vec!["user_created".to_string(), "role_assigned".to_string()],
            },
            ComplianceControl {
                id: "CC6.3".to_string(),
                name: "Access Removal".to_string(),
                description: "The entity removes access to protected information assets when appropriate".to_string(),
                framework: ComplianceFramework::Soc2,
                category: "Logical and Physical Access Controls".to_string(),
                evidence_categories: vec![AuditCategory::UserManagement],
                evidence_actions: vec!["user_deleted".to_string(), "user_disabled".to_string(), "role_revoked".to_string()],
            },
            ComplianceControl {
                id: "CC7.1".to_string(),
                name: "System Operations".to_string(),
                description: "The entity uses detection and monitoring procedures to identify changes to configurations".to_string(),
                framework: ComplianceFramework::Soc2,
                category: "System Operations".to_string(),
                evidence_categories: vec![AuditCategory::Configuration, AuditCategory::System],
                evidence_actions: vec!["config_updated".to_string(), "config_created".to_string()],
            },
            ComplianceControl {
                id: "CC7.2".to_string(),
                name: "Security Monitoring".to_string(),
                description: "The entity monitors system components for anomalies".to_string(),
                framework: ComplianceFramework::Soc2,
                category: "System Operations".to_string(),
                evidence_categories: vec![AuditCategory::Security],
                evidence_actions: vec!["suspicious_activity".to_string(), "security_violation".to_string()],
            },
        ];

        for control in controls {
            self.controls.insert(control.id.clone(), control);
        }
    }

    fn load_gdpr_controls(&mut self) {
        let controls = vec![
            ComplianceControl {
                id: "GDPR-30".to_string(),
                name: "Records of Processing Activities".to_string(),
                description: "Maintain records of data processing activities".to_string(),
                framework: ComplianceFramework::Gdpr,
                category: "Documentation".to_string(),
                evidence_categories: vec![AuditCategory::DataTransfer],
                evidence_actions: vec!["data_exported".to_string(), "data_imported".to_string()],
            },
            ComplianceControl {
                id: "GDPR-32".to_string(),
                name: "Security of Processing".to_string(),
                description: "Implement appropriate technical and organizational measures".to_string(),
                framework: ComplianceFramework::Gdpr,
                category: "Security".to_string(),
                evidence_categories: vec![AuditCategory::Authentication, AuditCategory::Security],
                evidence_actions: vec!["login".to_string(), "login_failed".to_string()],
            },
            ComplianceControl {
                id: "GDPR-33".to_string(),
                name: "Data Breach Notification".to_string(),
                description: "Notify supervisory authority of data breaches".to_string(),
                framework: ComplianceFramework::Gdpr,
                category: "Breach Response".to_string(),
                evidence_categories: vec![AuditCategory::Security],
                evidence_actions: vec!["data_breach".to_string(), "security_violation".to_string()],
            },
        ];

        for control in controls {
            self.controls.insert(control.id.clone(), control);
        }
    }

    /// Get a control by ID.
    pub fn get(&self, id: &str) -> Option<&ComplianceControl> {
        self.controls.get(id)
    }

    /// Get all controls for a framework.
    pub fn by_framework(&self, framework: ComplianceFramework) -> Vec<&ComplianceControl> {
        self.controls
            .values()
            .filter(|c| c.framework == framework)
            .collect()
    }

    /// Add a custom control.
    pub fn add(&mut self, control: ComplianceControl) {
        self.controls.insert(control.id.clone(), control);
    }
}

impl Default for ControlLibrary {
    fn default() -> Self {
        Self::new()
    }
}