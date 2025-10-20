use no_downtime_service::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_circuit_breaker_integration() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(100),
        success_threshold: 1,
    };
    let cb = CircuitBreaker::with_config(config);
    
    // Should start in closed state
    assert_eq!(cb.state(), CircuitState::Closed);
    
    // Successful call
    let result: Result<i32, CircuitBreakerError<&str>> = cb.call(|| async { Ok(42) }).await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(cb.state(), CircuitState::Closed);
    
    // One failure
    let result: Result<i32, CircuitBreakerError<&str>> = cb.call(|| async { Err("error") }).await;
    assert!(result.is_err());
    assert_eq!(cb.state(), CircuitState::Closed); // Still closed after one failure
    
    // Second failure opens the circuit
    let result: Result<i32, CircuitBreakerError<&str>> = cb.call(|| async { Err("error") }).await;
    assert!(result.is_err());
    assert_eq!(cb.state(), CircuitState::Open);
    
    // Immediate failure when circuit is open
    let result: Result<i32, CircuitBreakerError<&str>> = cb.call(|| async { Ok(42) }).await;
    assert!(matches!(result, Err(CircuitBreakerError::OpenCircuit)));
    
    // Wait for timeout to move to half-open
    sleep(Duration::from_millis(150)).await;
    cb.check_timeout().await;
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    
    // Success in half-open closes the circuit
    let result: Result<i32, CircuitBreakerError<&str>> = cb.call(|| async { Ok(42) }).await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_force_operations() {
    let cb = CircuitBreaker::new();
    
    // Force open
    cb.force_open();
    assert_eq!(cb.state(), CircuitState::Open);
    
    // Force close
    cb.force_close();
    assert_eq!(cb.state(), CircuitState::Closed);
}