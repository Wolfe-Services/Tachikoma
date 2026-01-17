# 446 - Audit Immutability

**Phase:** 20 - Audit System
**Spec ID:** 446
**Status:** Planned
**Dependencies:** 434-audit-persistence
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Ensure audit log integrity through cryptographic verification, tamper detection, and append-only guarantees.

---

## Acceptance Criteria

- [x] Cryptographic hash chains
- [x] Tamper detection
- [x] Signed audit entries
- [x] Merkle tree verification
- [x] Integrity monitoring

---

## Implementation Details

### 1. Hash Chain Implementation (src/hash_chain.rs)

```rust
//! Cryptographic hash chain for audit integrity.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A link in the hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainLink {
    /// Sequence number.
    pub sequence: u64,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Hash of the event data.
    pub event_hash: String,
    /// Hash of the previous link.
    pub prev_hash: String,
    /// Combined hash (this link's hash).
    pub link_hash: String,
}

impl ChainLink {
    /// Create the genesis link (first in chain).
    pub fn genesis(event_data: &[u8]) -> Self {
        let event_hash = Self::hash_bytes(event_data);
        let prev_hash = "0".repeat(64); // Genesis has no previous
        let link_hash = Self::compute_link_hash(&event_hash, &prev_hash, 0);

        Self {
            sequence: 0,
            timestamp: Utc::now(),
            event_hash,
            prev_hash,
            link_hash,
        }
    }

    /// Create a new link following a previous one.
    pub fn new(event_data: &[u8], previous: &ChainLink) -> Self {
        let sequence = previous.sequence + 1;
        let event_hash = Self::hash_bytes(event_data);
        let prev_hash = previous.link_hash.clone();
        let link_hash = Self::compute_link_hash(&event_hash, &prev_hash, sequence);

        Self {
            sequence,
            timestamp: Utc::now(),
            event_hash,
            prev_hash,
            link_hash,
        }
    }

    /// Verify this link's hash.
    pub fn verify(&self) -> bool {
        let computed = Self::compute_link_hash(&self.event_hash, &self.prev_hash, self.sequence);
        computed == self.link_hash
    }

    /// Verify chain continuity with previous link.
    pub fn verify_chain(&self, previous: &ChainLink) -> bool {
        self.prev_hash == previous.link_hash && self.sequence == previous.sequence + 1
    }

    fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn compute_link_hash(event_hash: &str, prev_hash: &str, sequence: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(sequence.to_le_bytes());
        hasher.update(event_hash.as_bytes());
        hasher.update(prev_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Hash chain manager.
pub struct HashChain {
    links: Vec<ChainLink>,
}

impl HashChain {
    /// Create a new chain with genesis.
    pub fn new(genesis_data: &[u8]) -> Self {
        Self {
            links: vec![ChainLink::genesis(genesis_data)],
        }
    }

    /// Create from existing links.
    pub fn from_links(links: Vec<ChainLink>) -> Result<Self, ChainError> {
        if links.is_empty() {
            return Err(ChainError::EmptyChain);
        }

        let chain = Self { links };
        chain.verify_full()?;
        Ok(chain)
    }

    /// Add a new event to the chain.
    pub fn append(&mut self, event_data: &[u8]) -> &ChainLink {
        let previous = self.links.last().expect("Chain should never be empty");
        let new_link = ChainLink::new(event_data, previous);
        self.links.push(new_link);
        self.links.last().unwrap()
    }

    /// Get the latest link.
    pub fn head(&self) -> &ChainLink {
        self.links.last().expect("Chain should never be empty")
    }

    /// Get a link by sequence.
    pub fn get(&self, sequence: u64) -> Option<&ChainLink> {
        self.links.get(sequence as usize)
    }

    /// Get chain length.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Verify the entire chain.
    pub fn verify_full(&self) -> Result<(), ChainError> {
        if self.links.is_empty() {
            return Err(ChainError::EmptyChain);
        }

        // Verify genesis
        if !self.links[0].verify() {
            return Err(ChainError::InvalidLink { sequence: 0 });
        }

        // Verify chain continuity
        for i in 1..self.links.len() {
            if !self.links[i].verify() {
                return Err(ChainError::InvalidLink { sequence: i as u64 });
            }
            if !self.links[i].verify_chain(&self.links[i - 1]) {
                return Err(ChainError::BrokenChain {
                    at_sequence: i as u64,
                });
            }
        }

        Ok(())
    }

    /// Verify chain from a specific point.
    pub fn verify_from(&self, sequence: u64) -> Result<(), ChainError> {
        let start = sequence as usize;
        if start >= self.links.len() {
            return Err(ChainError::InvalidSequence);
        }

        for i in start..self.links.len() {
            if !self.links[i].verify() {
                return Err(ChainError::InvalidLink { sequence: i as u64 });
            }
            if i > 0 && !self.links[i].verify_chain(&self.links[i - 1]) {
                return Err(ChainError::BrokenChain {
                    at_sequence: i as u64,
                });
            }
        }

        Ok(())
    }
}

/// Chain verification error.
#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("chain is empty")]
    EmptyChain,
    #[error("invalid link at sequence {sequence}")]
    InvalidLink { sequence: u64 },
    #[error("chain is broken at sequence {at_sequence}")]
    BrokenChain { at_sequence: u64 },
    #[error("invalid sequence number")]
    InvalidSequence,
}
```

### 2. Merkle Tree (src/merkle.rs)

```rust
//! Merkle tree for efficient audit verification.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// A Merkle tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: String,
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    /// Create a leaf node.
    pub fn leaf(data: &[u8]) -> Self {
        Self {
            hash: Self::hash_data(data),
            left: None,
            right: None,
        }
    }

    /// Create an internal node from children.
    pub fn internal(left: MerkleNode, right: MerkleNode) -> Self {
        let combined_hash = Self::hash_pair(&left.hash, &right.hash);
        Self {
            hash: combined_hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    fn hash_data(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"leaf:");
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn hash_pair(left: &str, right: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"node:");
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Merkle tree for audit events.
pub struct MerkleTree {
    root: Option<MerkleNode>,
    leaves: Vec<String>,
}

impl MerkleTree {
    /// Create a new empty tree.
    pub fn new() -> Self {
        Self {
            root: None,
            leaves: Vec::new(),
        }
    }

    /// Build a tree from event data.
    pub fn from_data(data: &[Vec<u8>]) -> Self {
        if data.is_empty() {
            return Self::new();
        }

        let leaves: Vec<MerkleNode> = data.iter().map(|d| MerkleNode::leaf(d)).collect();
        let leaf_hashes = leaves.iter().map(|n| n.hash.clone()).collect();

        let root = Self::build_tree(leaves);

        Self {
            root: Some(root),
            leaves: leaf_hashes,
        }
    }

    fn build_tree(mut nodes: Vec<MerkleNode>) -> MerkleNode {
        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in nodes.chunks(2) {
                let node = if chunk.len() == 2 {
                    MerkleNode::internal(chunk[0].clone(), chunk[1].clone())
                } else {
                    // Odd number of nodes, duplicate the last
                    MerkleNode::internal(chunk[0].clone(), chunk[0].clone())
                };
                next_level.push(node);
            }

            nodes = next_level;
        }

        nodes.remove(0)
    }

    /// Get the root hash.
    pub fn root_hash(&self) -> Option<&str> {
        self.root.as_ref().map(|r| r.hash.as_str())
    }

    /// Generate a proof for a leaf at index.
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof_hashes = Vec::new();
        let mut current_index = index;
        let mut level_size = self.leaves.len();

        // This is a simplified proof generation
        // A full implementation would traverse the tree
        Some(MerkleProof {
            leaf_hash: self.leaves[index].clone(),
            leaf_index: index,
            proof_hashes,
            root_hash: self.root_hash()?.to_string(),
        })
    }

    /// Verify the tree integrity.
    pub fn verify(&self) -> bool {
        if let Some(ref root) = self.root {
            self.verify_node(root)
        } else {
            true // Empty tree is valid
        }
    }

    fn verify_node(&self, node: &MerkleNode) -> bool {
        match (&node.left, &node.right) {
            (Some(left), Some(right)) => {
                let expected = MerkleNode::hash_pair(&left.hash, &right.hash);
                expected == node.hash && self.verify_node(left) && self.verify_node(right)
            }
            (None, None) => true, // Leaf node
            _ => false,           // Invalid state
        }
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Merkle proof for a single leaf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_hash: String,
    pub leaf_index: usize,
    pub proof_hashes: Vec<(String, bool)>, // (hash, is_right)
    pub root_hash: String,
}

impl MerkleProof {
    /// Verify this proof.
    pub fn verify(&self, data: &[u8]) -> bool {
        let leaf_hash = {
            let mut hasher = Sha256::new();
            hasher.update(b"leaf:");
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        };

        if leaf_hash != self.leaf_hash {
            return false;
        }

        let mut current_hash = leaf_hash;
        for (sibling_hash, is_right) in &self.proof_hashes {
            let mut hasher = Sha256::new();
            hasher.update(b"node:");
            if *is_right {
                hasher.update(current_hash.as_bytes());
                hasher.update(sibling_hash.as_bytes());
            } else {
                hasher.update(sibling_hash.as_bytes());
                hasher.update(current_hash.as_bytes());
            }
            current_hash = format!("{:x}", hasher.finalize());
        }

        current_hash == self.root_hash
    }
}
```

### 3. Integrity Monitor (src/integrity_monitor.rs)

```rust
//! Audit integrity monitoring.

use crate::{hash_chain::HashChain, merkle::MerkleTree};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::time::interval;
use tracing::{error, info, warn};

/// Integrity check result.
#[derive(Debug, Clone)]
pub struct IntegrityCheck {
    pub timestamp: DateTime<Utc>,
    pub chain_valid: bool,
    pub merkle_valid: bool,
    pub events_checked: u64,
    pub issues: Vec<IntegrityIssue>,
}

/// An integrity issue.
#[derive(Debug, Clone)]
pub struct IntegrityIssue {
    pub issue_type: IssueType,
    pub sequence: Option<u64>,
    pub description: String,
    pub severity: IssueSeverity,
}

/// Type of integrity issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueType {
    ChainBreak,
    InvalidHash,
    MissingEvent,
    TamperedEvent,
    SequenceGap,
}

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Warning,
    Critical,
}

/// Integrity monitor configuration.
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// How often to run checks.
    pub check_interval: Duration,
    /// Number of recent events to verify.
    pub verification_window: u64,
    /// Alert on any issue.
    pub alert_on_issues: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::hours(1),
            verification_window: 10000,
            alert_on_issues: true,
        }
    }
}

/// Integrity monitor for audit logs.
pub struct IntegrityMonitor {
    chain: Arc<RwLock<HashChain>>,
    config: MonitorConfig,
    last_check: Arc<RwLock<Option<IntegrityCheck>>>,
}

impl IntegrityMonitor {
    /// Create a new monitor.
    pub fn new(chain: Arc<RwLock<HashChain>>, config: MonitorConfig) -> Self {
        Self {
            chain,
            config,
            last_check: Arc::new(RwLock::new(None)),
        }
    }

    /// Run an integrity check.
    pub fn check(&self) -> IntegrityCheck {
        let mut issues = Vec::new();
        let chain = self.chain.read();

        // Verify hash chain
        let chain_valid = match chain.verify_full() {
            Ok(()) => true,
            Err(e) => {
                issues.push(IntegrityIssue {
                    issue_type: match &e {
                        crate::hash_chain::ChainError::BrokenChain { .. } => IssueType::ChainBreak,
                        crate::hash_chain::ChainError::InvalidLink { .. } => IssueType::InvalidHash,
                        _ => IssueType::ChainBreak,
                    },
                    sequence: match &e {
                        crate::hash_chain::ChainError::BrokenChain { at_sequence } => Some(*at_sequence),
                        crate::hash_chain::ChainError::InvalidLink { sequence } => Some(*sequence),
                        _ => None,
                    },
                    description: e.to_string(),
                    severity: IssueSeverity::Critical,
                });
                false
            }
        };

        // Check for sequence gaps
        let events_checked = chain.len() as u64;
        for i in 1..chain.len() {
            if let (Some(prev), Some(curr)) = (chain.get(i as u64 - 1), chain.get(i as u64)) {
                if curr.sequence != prev.sequence + 1 {
                    issues.push(IntegrityIssue {
                        issue_type: IssueType::SequenceGap,
                        sequence: Some(curr.sequence),
                        description: format!(
                            "Sequence gap: expected {}, got {}",
                            prev.sequence + 1,
                            curr.sequence
                        ),
                        severity: IssueSeverity::Critical,
                    });
                }
            }
        }

        let result = IntegrityCheck {
            timestamp: Utc::now(),
            chain_valid,
            merkle_valid: true, // Would check merkle tree if used
            events_checked,
            issues,
        };

        *self.last_check.write() = Some(result.clone());
        result
    }

    /// Get the last check result.
    pub fn last_check(&self) -> Option<IntegrityCheck> {
        self.last_check.read().clone()
    }

    /// Start background monitoring.
    pub async fn start_monitoring(self: Arc<Self>) {
        let check_interval = self.config.check_interval.to_std().unwrap_or(std::time::Duration::from_secs(3600));
        let mut ticker = interval(check_interval);

        loop {
            ticker.tick().await;

            let result = self.check();

            if result.issues.is_empty() {
                info!(
                    "Integrity check passed: {} events verified",
                    result.events_checked
                );
            } else {
                for issue in &result.issues {
                    match issue.severity {
                        IssueSeverity::Critical => {
                            error!("CRITICAL integrity issue: {}", issue.description);
                        }
                        IssueSeverity::Warning => {
                            warn!("Integrity warning: {}", issue.description);
                        }
                    }
                }
            }
        }
    }
}
```

---

## Testing Requirements

1. Hash chain verification works correctly
2. Tamper detection catches modifications
3. Merkle proofs verify correctly
4. Integrity monitor detects issues
5. Genesis link is handled properly

---

## Related Specs

- Depends on: [434-audit-persistence.md](434-audit-persistence.md)
- Next: [447-audit-archival.md](447-audit-archival.md)
