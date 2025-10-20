//! Distributed Tracing Module
//!
//! This module implements basic tracing for the zero-downtime service.
//! It provides structured logging with trace context.

use tracing_subscriber::{filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize basic tracing
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // Create a filter for tracing
    let filter = filter::EnvFilter::try_from_default_env()
        .or_else(|_| filter::EnvFilter::try_new("info"))
        .unwrap();

    // Use the tracing subscriber with JSON formatting
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init()?;

    Ok(())
}

/// Shutdown tracing gracefully
pub fn shutdown_tracing() {
    // For basic tracing, there's no specific shutdown needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_initialization() {
        // Initialize tracing
        let result = init_tracing();
        assert!(result.is_ok());
    }
}