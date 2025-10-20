//! Integration tests for the distributed tracing module

use no_downtime_service::tracing;

#[test]
fn test_tracing_initialization() {
    // Initialize tracing
    let result = tracing::init_tracing();
    assert!(result.is_ok());
    
    // Shutdown tracing
    tracing::shutdown_tracing();
}