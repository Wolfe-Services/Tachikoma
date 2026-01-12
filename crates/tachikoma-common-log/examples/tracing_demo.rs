//! Example demonstrating distributed tracing capabilities.

use tachikoma_common_log::{
    spans::{mission_span, backend_span, file_span, instrument_future, Timer, instrument},
    LogConfig, init,
};
use tracing::{info, error};

#[instrument(skip(_config), err)]
async fn process_mission(mission_id: &str, _config: &str) -> Result<(), &'static str> {
    info!("Starting mission processing");
    
    // Simulate file operations
    let file_span = file_span("read", "config.yaml");
    let file_future = async {
        info!("Reading configuration file");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok::<(), &'static str>(())
    };
    instrument_future(file_future, file_span).await?;
    
    // Simulate backend operations
    let backend_span = backend_span("database", "query");
    let db_future = async {
        info!("Querying database for mission data");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok::<(), &'static str>(())
    };
    instrument_future(db_future, backend_span).await?;
    
    info!("Mission processing completed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with tracing support
    let config = LogConfig {
        span_events: true,
        source_location: true,
        ..Default::default()
    };
    init(config)?;
    
    info!("Starting tracing demo");
    
    // Create a mission span and run operations within it
    let mission = mission_span("demo-mission-001");
    let mission_future = async {
        info!("Mission span active");
        
        // Time a critical operation
        let timer = Timer::start("critical_operation");
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        timer.finish();
        
        // Process mission with instrumented function
        match process_mission("demo-mission-001", "config").await {
            Ok(_) => info!("Mission completed successfully"),
            Err(e) => error!("Mission failed: {}", e),
        }
        
        Ok::<(), &'static str>(())
    };
    
    // Run the entire mission within the span
    instrument_future(mission_future, mission).await?;
    
    info!("Tracing demo completed");
    Ok(())
}