//! TLS implementation tests

use no_downtime_service::config::{Config, TlsConfig};

#[test]
fn test_tls_config_creation() {
    let tls_config = TlsConfig {
        cert_path: "cert.pem".to_string(),
        key_path: "key.pem".to_string(),
    };
    
    assert_eq!(tls_config.cert_path, "cert.pem");
    assert_eq!(tls_config.key_path, "key.pem");
}

#[test]
fn test_config_with_tls() {
    // This test just verifies that the Config struct can be created with TLS config
    // In a real test, we would need actual certificate files
    let config = Config {
        bind_address: "0.0.0.0:3000".parse().unwrap(),
        shutdown_timeout: 30,
        tls: Some(TlsConfig {
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
        }),
    };
    
    assert!(config.tls.is_some());
    assert_eq!(config.tls.unwrap().cert_path, "cert.pem");
}

// Test that shows TLS config can be None (no TLS)
#[test]
fn test_config_without_tls() {
    let config = Config {
        bind_address: "0.0.0.0:3000".parse().unwrap(),
        shutdown_timeout: 30,
        tls: None,
    };
    
    assert!(config.tls.is_none());
}