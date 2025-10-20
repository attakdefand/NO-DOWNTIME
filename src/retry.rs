//! Retry mechanism implementation for resilience patterns
//!
//! This module implements retry mechanisms with exponential backoff
//! and jitter to handle transient failures.

use rand::{thread_rng, Rng};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    /// Jitter factor (0.0 to 1.0)
    pub jitter: f64,
    /// Whether to use exponential backoff
    pub exponential: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            jitter: 0.1,
            exponential: true,
        }
    }
}

/// Error type for retry operations
#[derive(Debug)]
pub enum RetryError<E> {
    /// All retry attempts failed
    AllAttemptsFailed(Vec<E>),
    /// Operation was cancelled
    Cancelled,
}

impl<E> std::fmt::Display for RetryError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::AllAttemptsFailed(_) => write!(f, "All retry attempts failed"),
            RetryError::Cancelled => write!(f, "Operation was cancelled"),
        }
    }
}

impl<E> std::error::Error for RetryError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RetryError::AllAttemptsFailed(_) => None,
            RetryError::Cancelled => None,
        }
    }
}

/// Retry mechanism implementation
pub struct RetryMechanism {
    config: RetryConfig,
}

impl RetryMechanism {
    /// Create a new retry mechanism with default configuration
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Create a new retry mechanism with custom configuration
    pub fn with_config(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    pub async fn retry<F, Fut, T, E>(&self, mut f: F) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempts = 0;
        let mut errors = Vec::new();
        let mut delay = self.config.initial_delay;

        loop {
            attempts += 1;
            
            match f().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    errors.push(error);
                    
                    // If we've reached the maximum attempts, return all errors
                    if attempts >= self.config.max_attempts {
                        return Err(RetryError::AllAttemptsFailed(errors));
                    }
                    
                    // Calculate next delay with exponential backoff
                    if self.config.exponential && attempts > 1 {
                        let multiplier = self.config.multiplier.powi(attempts as i32 - 1);
                        let next_delay = self.config.initial_delay.mul_f64(multiplier);
                        delay = next_delay.min(self.config.max_delay);
                    }
                    
                    // Add jitter
                    if self.config.jitter > 0.0 {
                        let jitter_range = (delay.as_millis() as f64 * self.config.jitter) as u64;
                        if jitter_range > 0 {
                            let jitter = thread_rng().gen_range(0..=jitter_range);
                            delay += Duration::from_millis(jitter);
                        }
                    }
                    
                    // Sleep for the calculated delay
                    sleep(delay).await;
                }
            }
        }
    }

    /// Execute a function with retry logic and a cancellation token
    pub async fn retry_with_cancel<F, Fut, T, E, C, CFut>(
        &self,
        mut f: F,
        cancel: C,
    ) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
        C: Fn() -> CFut,
        CFut: Future<Output = bool>,
    {
        let mut attempts = 0;
        let mut errors = Vec::new();
        let mut delay = self.config.initial_delay;

        loop {
            // Check for cancellation
            if cancel().await {
                return Err(RetryError::Cancelled);
            }
            
            attempts += 1;
            
            match f().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    errors.push(error);
                    
                    // If we've reached the maximum attempts, return all errors
                    if attempts >= self.config.max_attempts {
                        return Err(RetryError::AllAttemptsFailed(errors));
                    }
                    
                    // Check for cancellation again before sleeping
                    if cancel().await {
                        return Err(RetryError::Cancelled);
                    }
                    
                    // Calculate next delay with exponential backoff
                    if self.config.exponential && attempts > 1 {
                        let multiplier = self.config.multiplier.powi(attempts as i32 - 1);
                        let next_delay = self.config.initial_delay.mul_f64(multiplier);
                        delay = next_delay.min(self.config.max_delay);
                    }
                    
                    // Add jitter
                    if self.config.jitter > 0.0 {
                        let jitter_range = (delay.as_millis() as f64 * self.config.jitter) as u64;
                        if jitter_range > 0 {
                            let jitter = thread_rng().gen_range(0..=jitter_range);
                            delay += Duration::from_millis(jitter);
                        }
                    }
                    
                    // Sleep for the calculated delay
                    sleep(delay).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let retry = RetryMechanism::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let result: Result<i32, RetryError<&'static str>> = retry.retry(|| async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(42)
        }).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let retry = RetryMechanism::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let result: Result<i32, RetryError<&'static str>> = retry.retry(|| async {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err("temporary error")
            } else {
                Ok(42)
            }
        }).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_all_attempts_fail() {
        let config = RetryConfig {
            max_attempts: 3,
            ..Default::default()
        };
        let retry = RetryMechanism::with_config(config);
        
        let result: Result<i32, RetryError<&'static str>> = retry.retry(|| async {
            Err("permanent error")
        }).await;
        
        assert!(matches!(result, Err(RetryError::AllAttemptsFailed(errors)) if errors.len() == 3));
    }

    #[tokio::test]
    async fn test_retry_with_cancel() {
        let retry = RetryMechanism::new();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        let cancel_counter = Arc::new(AtomicU32::new(0));
        let cancel_counter_clone = cancel_counter.clone();
        
        let result: Result<i32, RetryError<&'static str>> = retry.retry_with_cancel(
            || async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err("temporary error")
            },
            || async {
                let count = cancel_counter_clone.fetch_add(1, Ordering::SeqCst);
                count >= 1 // Cancel after 1 check
            }
        ).await;
        
        assert!(matches!(result, Err(RetryError::Cancelled)));
        assert!(counter.load(Ordering::SeqCst) >= 1);
        assert!(cancel_counter.load(Ordering::SeqCst) >= 1);
    }
}