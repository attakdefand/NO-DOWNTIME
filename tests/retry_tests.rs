use no_downtime_service::retry::{RetryMechanism, RetryConfig, RetryError};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_retry_integration() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        multiplier: 2.0,
        jitter: 0.0,
        exponential: true,
    };
    let retry = RetryMechanism::with_config(config);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    
    let result: Result<i32, RetryError<&str>> = retry.retry(|| async {
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
async fn test_retry_all_failures() {
    let config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        multiplier: 2.0,
        jitter: 0.0,
        exponential: true,
    };
    let retry = RetryMechanism::with_config(config);
    
    let result: Result<i32, RetryError<&str>> = retry.retry(|| async {
        Err("permanent error")
    }).await;
    
    assert!(matches!(result, Err(RetryError::AllAttemptsFailed(errors)) if errors.len() == 2));
}

#[tokio::test]
async fn test_retry_cancel() {
    let retry = RetryMechanism::new();
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let cancel_counter = Arc::new(AtomicU32::new(0));
    let cancel_counter_clone = cancel_counter.clone();
    
    let result: Result<i32, RetryError<&str>> = retry.retry_with_cancel(
        || async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err("temporary error")
        },
        || async { 
            let count = cancel_counter_clone.fetch_add(1, Ordering::SeqCst);
            count >= 1 // Cancel after first check
        }
    ).await;
    
    assert!(matches!(result, Err(RetryError::Cancelled)));
    // The function should be called once before cancellation
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert!(cancel_counter.load(Ordering::SeqCst) >= 1);
}