//! Integration tests for the metrics collection module

use no_downtime_service::metrics::Metrics;
use prometheus::Encoder;

#[test]
fn test_metrics_initialization() {
    let metrics = Metrics::new();
    assert!(metrics.is_ok());
}

#[test]
fn test_metrics_collection() {
    let metrics = Metrics::new().unwrap();
    
    // Test HTTP requests counter
    metrics.increment_http_requests("GET", "/api/users", "200");
    metrics.increment_http_requests("POST", "/api/users", "201");
    
    // Test active connections gauge
    metrics.increment_active_connections();
    metrics.increment_active_connections();
    assert_eq!(metrics.active_connections.get(), 2.0);
    
    metrics.decrement_active_connections();
    assert_eq!(metrics.active_connections.get(), 1.0);
    
    metrics.set_active_connections(10.0);
    assert_eq!(metrics.active_connections.get(), 10.0);
    
    // Test request duration histogram
    metrics.record_request_duration("GET", "/api/users", 0.05);
    metrics.record_request_duration("POST", "/api/users", 0.15);
    
    // Test errors counter
    metrics.increment_errors("database", "/api/users");
    metrics.increment_errors("validation", "/api/users");
    
    // Verify metrics are collected
    let metric_families = metrics.registry().gather();
    assert!(!metric_families.is_empty());
    
    // Test that we can encode metrics to text format
    let mut buffer = Vec::new();
    let encoder = prometheus::TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("no_downtime_http_requests_total"));
    assert!(output.contains("no_downtime_active_connections"));
    assert!(output.contains("no_downtime_request_duration_seconds"));
    assert!(output.contains("no_downtime_errors_total"));
}

#[test]
fn test_metrics_labels() {
    let metrics = Metrics::new().unwrap();
    
    // Test that metrics with different labels are tracked separately
    metrics.increment_http_requests("GET", "/api/users", "200");
    metrics.increment_http_requests("GET", "/api/users", "404");
    metrics.increment_http_requests("POST", "/api/users", "201");
    
    let metric_families = metrics.registry().gather();
    let http_requests_family = metric_families.iter()
        .find(|m| m.get_name() == "no_downtime_http_requests_total")
        .expect("Should have http_requests_total metric");
    
    // Should have 3 different metrics for the different label combinations
    assert_eq!(http_requests_family.get_metric().len(), 3);
}