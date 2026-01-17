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

impl Default for HashChain {
    fn default() -> Self {
        Self::new(b"genesis")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_link() {
        let link = ChainLink::genesis(b"test data");
        assert_eq!(link.sequence, 0);
        assert!(link.verify());
        assert_eq!(link.prev_hash.len(), 64); // SHA256 hex length
        assert!(link.prev_hash.chars().all(|c| c == '0'));
    }

    #[test]
    fn test_chain_creation() {
        let chain = HashChain::new(b"genesis");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.head().sequence, 0);
    }

    #[test]
    fn test_chain_append() {
        let mut chain = HashChain::new(b"genesis");
        let link = chain.append(b"event 1");
        assert_eq!(link.sequence, 1);
        assert!(link.verify());
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_chain_verification() {
        let mut chain = HashChain::new(b"genesis");
        chain.append(b"event 1");
        chain.append(b"event 2");
        assert!(chain.verify_full().is_ok());
    }

    #[test]
    fn test_chain_continuity() {
        let link1 = ChainLink::genesis(b"event 1");
        let link2 = ChainLink::new(b"event 2", &link1);
        assert!(link2.verify_chain(&link1));
    }

    #[test]
    fn test_invalid_chain() {
        let link1 = ChainLink::genesis(b"event 1");
        let mut link2 = ChainLink::new(b"event 2", &link1);
        link2.prev_hash = "invalid".to_string();
        assert!(!link2.verify_chain(&link1));
    }
}