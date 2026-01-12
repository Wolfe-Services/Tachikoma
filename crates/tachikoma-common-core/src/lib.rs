//! Tachikoma common core types and utilities.
//!
//! This crate provides the fundamental types, errors, and utilities
//! used throughout the Tachikoma project.
//!
//! # Examples
//!
//! ```rust
//! use tachikoma_common_core::Error;
//! 
//! // Work with Tachikoma errors
//! let error = Error::new("Something went wrong");
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

pub mod error;
pub mod types;

pub use error::Error;
pub use types::*;
