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
        if index >= self.leaves.len() || self.root.is_none() {
            return None;
        }

        let mut proof_hashes = Vec::new();
        let mut current_index = index;
        let mut level_size = self.leaves.len();

        // Build proof by traversing tree levels
        while level_size > 1 {
            let is_right = current_index % 2 == 1;
            let sibling_index = if is_right {
                current_index - 1
            } else {
                (current_index + 1).min(level_size - 1)
            };

            if current_index != sibling_index {
                if level_size <= self.leaves.len() {
                    // We're at leaf level
                    proof_hashes.push((self.leaves[sibling_index].clone(), !is_right));
                }
            }

            current_index = current_index / 2;
            level_size = (level_size + 1) / 2;
        }

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

    /// Get the number of leaves.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Check if tree is empty.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_node_leaf() {
        let node = MerkleNode::leaf(b"test data");
        assert!(!node.hash.is_empty());
        assert!(node.left.is_none());
        assert!(node.right.is_none());
    }

    #[test]
    fn test_merkle_tree_empty() {
        let tree = MerkleTree::new();
        assert!(tree.is_empty());
        assert!(tree.root_hash().is_none());
        assert!(tree.verify());
    }

    #[test]
    fn test_merkle_tree_single_leaf() {
        let data = vec![vec![1, 2, 3, 4]];
        let tree = MerkleTree::from_data(&data);
        assert_eq!(tree.len(), 1);
        assert!(tree.root_hash().is_some());
        assert!(tree.verify());
    }

    #[test]
    fn test_merkle_tree_multiple_leaves() {
        let data = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
        ];
        let tree = MerkleTree::from_data(&data);
        assert_eq!(tree.len(), 3);
        assert!(tree.root_hash().is_some());
        assert!(tree.verify());
    }

    #[test]
    fn test_merkle_proof_generation() {
        let data = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
        ];
        let tree = MerkleTree::from_data(&data);
        
        let proof = tree.proof(0);
        assert!(proof.is_some());
        
        let proof = proof.unwrap();
        assert!(proof.verify(&[1, 2, 3, 4]));
        assert!(!proof.verify(&[9, 9, 9, 9]));
    }

    #[test]
    fn test_merkle_proof_invalid_index() {
        let data = vec![vec![1, 2, 3, 4]];
        let tree = MerkleTree::from_data(&data);
        
        assert!(tree.proof(1).is_none());
        assert!(tree.proof(100).is_none());
    }
}