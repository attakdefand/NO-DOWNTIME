//! Metrics Collection Module
//!
//! This module implements Prometheus metrics collection for the zero-downtime service.
//! It provides counters, gauges, and histograms for monitoring service performance.

use std::sync::Arc;
use prometheus::{CounterVec, Gauge, HistogramVec, Opts, Registry, HistogramOpts};

/// Metrics collection struct
#[derive(Clone)]
pub struct Metrics {
    /// Registry for storing metrics
    registry: Arc<Registry>,
    
    /// Counter for HTTP requests
    pub http_requests_total: CounterVec,
    
    /// Gauge for active connections
    pub active_connections: Gauge,
    
    /// Histogram for request duration
    pub request_duration_seconds: HistogramVec,
    
    /// Counter for errors
    pub errors_total: CounterVec,
}

impl Metrics {
    /// Create a new metrics instance
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        
        // HTTP requests counter
        let http_requests_total_opts = Opts::new("no_downtime_http_requests_total", "Total number of HTTP requests");
        let http_requests_total = CounterVec::new(http_requests_total_opts, &["method", "endpoint", "status"])?;
        registry.register(Box::new(http_requests_total.clone()))?;
        
        // Active connections gauge
        let active_connections_opts = Opts::new("no_downtime_active_connections", "Number of active connections");
        let active_connections = Gauge::with_opts(active_connections_opts)?;
        registry.register(Box::new(active_connections.clone()))?;
        
        // Request duration histogram
        let request_duration_seconds_opts = HistogramOpts::new("no_downtime_request_duration_seconds", "Request duration in seconds");
        let request_duration_seconds = HistogramVec::new(
            request_duration_seconds_opts,
            &["method", "endpoint"]
        )?;
        registry.register(Box::new(request_duration_seconds.clone()))?;
        
        // Errors counter
        let errors_total_opts = Opts::new("no_downtime_errors_total", "Total number of errors");
        let errors_total = CounterVec::new(errors_total_opts, &["type", "endpoint"])?;
        registry.register(Box::new(errors_total.clone()))?;
        
        Ok(Metrics {
            registry,
            http_requests_total,
            active_connections,
            request_duration_seconds,
            errors_total,
        })
    }
    
    /// Get the metrics registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    
    /// Increment HTTP requests counter
    pub fn increment_http_requests(&self, method: &str, endpoint: &str, status: &str) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, status])
            .inc();
    }
    
    /// Increment active connections
    pub fn increment_active_connections(&self) {
        self.active_connections.inc();
    }
    
    /// Decrement active connections
    pub fn decrement_active_connections(&self) {
        self.active_connections.dec();
    }
    
    /// Set active connections
    pub fn set_active_connections(&self, count: f64) {
        self.active_connections.set(count);
    }
    
    /// Record request duration
    pub fn record_request_duration(&self, method: &str, endpoint: &str, duration: f64) {
        self.request_duration_seconds
            .with_label_values(&[method, endpoint])
            .observe(duration);
    }
    
    /// Increment errors counter
    pub fn increment_errors(&self, error_type: &str, endpoint: &str) {
        self.errors_total
            .with_label_values(&[error_type, endpoint])
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Encoder;
    
    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert!(metrics.is_ok());
    }
    
    #[test]
    fn test_http_requests_counter() {
        let metrics = Metrics::new().unwrap();
        metrics.increment_http_requests("GET", "/api/users", "200");
        
        // Check that the counter was incremented
        let metric_family = metrics.registry.gather();
        let http_requests_family = metric_family.iter()
            .find(|m| m.get_name() == "no_downtime_http_requests_total")
            .expect("Should have http_requests_total metric");
        
        assert_eq!(http_requests_family.get_metric().len(), 1);
    }
    
    #[test]
    fn test_active_connections_gauge() {
        let metrics = Metrics::new().unwrap();
        metrics.increment_active_connections();
        assert_eq!(metrics.active_connections.get(), 1.0);
        
        metrics.decrement_active_connections();
        assert_eq!(metrics.active_connections.get(), 0.0);
        
        metrics.set_active_connections(5.0);
        assert_eq!(metrics.active_connections.get(), 5.0);
    }
    
    #[test]
    fn test_request_duration_histogram() {
        let metrics = Metrics::new().unwrap();
        metrics.record_request_duration("GET", "/api/users", 0.1);
        
        // Check that the histogram was recorded
        let metric_family = metrics.registry.gather();
        let duration_family = metric_family.iter()
            .find(|m| m.get_name() == "no_downtime_request_duration_seconds")
            .expect("Should have request_duration_seconds metric");
        
        assert_eq!(duration_family.get_metric().len(), 1);
    }
    
    #[test]
    fn test_errors_counter() {
        let metrics = Metrics::new().unwrap();
        metrics.increment_errors("database", "/api/users");
        
        // Check that the counter was incremented
        let metric_family = metrics.registry.gather();
        let errors_family = metric_family.iter()
            .find(|m| m.get_name() == "no_downtime_errors_total")
            .expect("Should have errors_total metric");
        
        assert_eq!(errors_family.get_metric().len(), 1);
    }
    
    #[test]
    fn test_metrics_registry_output() {
        let metrics = Metrics::new().unwrap();
        metrics.increment_http_requests("GET", "/api/users", "200");
        
        let metric_families = metrics.registry.gather();
        let mut buffer = Vec::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("no_downtime_http_requests_total"));
    }
}