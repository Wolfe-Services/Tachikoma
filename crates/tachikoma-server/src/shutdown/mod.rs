//! Graceful shutdown handling.

pub mod coordinator;
pub mod handler;
pub mod hooks;
pub mod signal;

pub use coordinator::ShutdownCoordinator;
pub use handler::ShutdownHandler;
pub use hooks::ShutdownHook;
pub use signal::{setup_signal_handlers, shutdown_signal};