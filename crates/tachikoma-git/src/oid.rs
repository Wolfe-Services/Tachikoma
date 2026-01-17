//! Git object ID wrapper.

use git2::Oid as Git2Oid;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::fmt;
use std::str::FromStr;

/// Git object identifier (SHA-1 hash).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GitOid(Git2Oid);

impl GitOid {
    /// Create from git2 Oid.
    pub fn from_git2(oid: Git2Oid) -> Self {
        Self(oid)
    }

    /// Get the underlying git2 Oid.
    pub fn as_git2(&self) -> Git2Oid {
        self.0
    }

    /// Parse from hex string.
    pub fn from_hex(hex: &str) -> Result<Self, GitOidError> {
        Git2Oid::from_str(hex)
            .map(Self)
            .map_err(|_| GitOidError::InvalidHex(hex.to_string()))
    }

    /// Get as hex string.
    pub fn to_hex(&self) -> String {
        self.0.to_string()
    }

    /// Get short form (first 7 characters).
    pub fn short(&self) -> String {
        self.to_hex()[..7].to_string()
    }

    /// Check if this is a zero OID.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Create a zero OID.
    pub fn zero() -> Self {
        Self(Git2Oid::zero())
    }
}

impl fmt::Display for GitOid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for GitOid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GitOid({})", self.short())
    }
}

impl FromStr for GitOid {
    type Err = GitOidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Serialize for GitOid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for GitOid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Git OID error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GitOidError {
    #[error("invalid hex string: {0}")]
    InvalidHex(String),
}