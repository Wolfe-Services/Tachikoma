//! Example demonstrating the Tachikoma logging infrastructure.

use std::path::PathBuf;
use tachikoma_common_log::{debug, error, info, init, trace, warn, LogConfig, LogFormat, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing JSON Format with File Output ===");
    let config = LogConfig {
        level: LogLevel::Debug,
        format: LogFormat::Json,
        file_path: Some(PathBuf::from("/tmp/tachikoma-test.log")),
        timestamps: true,
        source_location: true,
        span_events: false,
    };

    init(config)?;

    // Test all log levels
    trace!("This is a trace message (should not appear with Debug level)");
    debug!("This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");

    println!("\n=== Environment-based Configuration Demo ===");
    let env_config = LogConfig::from_env();
    println!("Environment config: {:?}", env_config);

    println!("\nLog file should be created at: /tmp/tachikoma-test.log");

    Ok(())
}