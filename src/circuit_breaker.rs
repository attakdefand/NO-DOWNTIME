//! Circuit Breaker implementation for resilience patterns
//!
//! This module implements the circuit breaker pattern to prevent cascading failures
//! and provide fast failure mechanisms when dependent services are unavailable.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};


/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed and allowing requests
    Closed,
    /// Circuit is open and rejecting requests
    Open,
    /// Circuit is half-open, allowing limited requests to test service availability
    HalfOpen,
}

/// Configuration for the circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: usize,
    /// Timeout before attempting to close the circuit again
    pub timeout: Duration,
    /// Number of successful requests needed to close the circuit from half-open state
    pub success_threshold: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 1,
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Current state of the circuit
    state: Arc<Mutex<CircuitState>>,
    /// Number of failures
    failure_count: Arc<AtomicUsize>,
    /// Last time the circuit opened
    last_failure: Arc<Mutex<Option<Instant>>>,
    /// Number of successes in half-open state
    success_count: Arc<AtomicUsize>,
    /// Configuration
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            success_count: Arc::new(AtomicUsize::new(0)),
            config,
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        // Check if we can execute the request
        match self.state() {
            CircuitState::Open => {
                return Err(CircuitBreakerError::OpenCircuit);
            }
            CircuitState::HalfOpen => {
                // In half-open state, we only allow limited requests
                // For simplicity, we'll allow all requests but track success/failure
            }
            CircuitState::Closed => {
                // Circuit is closed, allow the request
            }
        }

        // Execute the function
        let result = f().await;

        // Update circuit breaker state based on result
        match result {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            }
            Err(error) => {
                self.on_error().await;
                Err(CircuitBreakerError::Inner(error))
            }
        }
    }

    /// Get the current state of the circuit breaker
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    /// Handle a successful request
    async fn on_success(&self) {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitState::Closed => {
                // Reset failure count
                self.failure_count.store(0, Ordering::Release);
            }
            CircuitState::Open => {
                // Should not happen, but if it does, ignore
            }
            CircuitState::HalfOpen => {
                // Increment success count
                let success_count = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;
                
                // If we've reached the success threshold, close the circuit
                if success_count >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Release);
                    self.success_count.store(0, Ordering::Release);
                }
            }
        }
    }

    /// Handle a failed request
    async fn on_error(&self) {
        // Increment failure count
        let failure_count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitState::Closed => {
                // If we've reached the failure threshold, open the circuit
                if failure_count >= self.config.failure_threshold {
                    *state = CircuitState::Open;
                    *self.last_failure.lock().unwrap() = Some(Instant::now());
                }
            }
            CircuitState::Open => {
                // Should not happen, but if it does, ignore
            }
            CircuitState::HalfOpen => {
                // Failed request in half-open state means service is still down
                // Re-open the circuit
                *state = CircuitState::Open;
                *self.last_failure.lock().unwrap() = Some(Instant::now());
                self.success_count.store(0, Ordering::Release);
            }
        }
    }

    /// Check if the circuit should attempt to close
    pub async fn check_timeout(&self) {
        let state = *self.state.lock().unwrap();
        
        if state == CircuitState::Open {
            let last_failure = *self.last_failure.lock().unwrap();
            
            if let Some(last_failure) = last_failure {
                if last_failure.elapsed() >= self.config.timeout {
                    // Move to half-open state
                    let mut state = self.state.lock().unwrap();
                    if *state == CircuitState::Open {
                        *state = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::Release);
                    }
                }
            }
        }
    }

    /// Force the circuit to open
    pub fn force_open(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Open;
        *self.last_failure.lock().unwrap() = Some(Instant::now());
    }

    /// Force the circuit to close
    pub fn force_close(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
        *self.last_failure.lock().unwrap() = None;
    }
}

/// Circuit breaker errors
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Inner error from the wrapped function
    Inner(E),
    /// Circuit is open and rejecting requests
    OpenCircuit,
}

impl<E> std::fmt::Display for CircuitBreakerError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Inner(e) => write!(f, "Inner error: {}", e),
            CircuitBreakerError::OpenCircuit => write!(f, "Circuit breaker is open"),
        }
    }
}

impl<E> std::error::Error for CircuitBreakerError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::Inner(e) => Some(e),
            CircuitBreakerError::OpenCircuit => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::new();
        
        // Circuit should start closed
        assert_eq!(cb.state(), CircuitState::Closed);
        
        // Successful call should keep circuit closed
        let result: Result<(), CircuitBreakerError<&'static str>> = cb.call(|| async { Ok(()) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(1),
            success_threshold: 1,
        };
        let cb = CircuitBreaker::with_config(config);
        
        // Fail 3 times to open the circuit
        for _ in 0..3 {
            let result: Result<(), CircuitBreakerError<&'static str>> = cb.call(|| async { Err("failure") }).await;
            assert!(result.is_err());
        }
        
        // Circuit should now be open
        assert_eq!(cb.state(), CircuitState::Open);
        
        // Next call should fail immediately
        let result: Result<(), CircuitBreakerError<&'static str>> = cb.call(|| async { Ok(()) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::OpenCircuit)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            success_threshold: 1,
        };
        let cb = CircuitBreaker::with_config(config);
        
        // Fail to open the circuit
        let result: Result<(), CircuitBreakerError<&'static str>> = cb.call(|| async { Err("failure") }).await;
        assert!(result.is_err());
        assert_eq!(cb.state(), CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Check timeout to move to half-open
        cb.check_timeout().await;
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            success_threshold: 1,
        };
        let cb = CircuitBreaker::with_config(config);
        
        // Fail to open the circuit
        let result: Result<(), CircuitBreakerError<&'static str>> = cb.call(|| async { Err("failure") }).await;
        assert!(result.is_err());
        assert_eq!(cb.state(), CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Check timeout to move to half-open
        cb.check_timeout().await;
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        
        // Success in half-open should close the circuit
        let result: Result<(), CircuitBreakerError<&'static str>> = cb.call(|| async { Ok(()) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}