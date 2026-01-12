//! Tachikoma CLI Library
//!
//! Core library components for the Tachikoma CLI.

pub mod cli;
pub mod commands;
pub mod error;
pub mod output;
pub mod prompts;

pub use error::CliError;