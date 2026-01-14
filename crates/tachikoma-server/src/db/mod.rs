//! Database connection management and utilities.
//!
//! This module provides comprehensive database functionality including:
//! - Connection pool configuration and management
//! - Health monitoring and metrics
//! - Query instrumentation and timeout handling
//! - Transaction helpers and retry logic
//! - Migration support

pub mod config;
pub mod health;
pub mod instrumentation;
pub mod migration;
pub mod pool;
pub mod transaction;

pub use config::DbConfig;
pub use health::{check_health, DbHealth};
pub use instrumentation::{QueryLogger, QueryTimer};
pub use migration::{check_migrations, run_migrations};
pub use pool::{create_pool, pool_stats, verify_connection, PoolStats};
pub use transaction::{begin_with_isolation, with_retry, with_transaction, IsolationLevel};