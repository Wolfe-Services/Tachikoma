//! Audit immutability and integrity verification.
//!
//! This crate provides cryptographic guarantees for audit log integrity
//! through hash chains, merkle trees, digital signatures, and continuous
//! integrity monitoring.

mod hash_chain;
mod merkle;
mod integrity_monitor;
mod signatures;

pub use hash_chain::{ChainLink, HashChain, ChainError};
pub use merkle::{MerkleTree, MerkleNode, MerkleProof};
pub use integrity_monitor::{
    IntegrityMonitor, IntegrityCheck, IntegrityIssue, IssueType, 
    IssueSeverity, MonitorConfig
};
pub use signatures::{
    AuditSigner, SignedAuditEntry, SignedChainLink, 
    SignatureError, KeyPair, VerificationResult
};

/// Re-export common types for convenience.
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};

/// Result type for immutability operations.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;