use no_downtime_service::rate_limiter::{RateLimiter, RateLimiterConfig, RateLimitingAlgorithm};

#[test]
fn test_rate_limiter_integration() {
    let config = RateLimiterConfig {
        algorithm: RateLimitingAlgorithm::TokenBucket {
            max_tokens: 5,
            tokens_per_second: 10.0,
        },
        per_client: false,
    };
    let limiter = RateLimiter::with_config(config);
    
    // Should allow first 5 requests
    for _ in 0..5 {
        assert!(limiter.allow(None));
    }
    
    // Should reject 6th request
    assert!(!limiter.allow(None));
}

#[test]
fn test_rate_limiter_per_client_integration() {
    let config = RateLimiterConfig {
        algorithm: RateLimitingAlgorithm::TokenBucket {
            max_tokens: 3,
            tokens_per_second: 10.0,
        },
        per_client: true,
    };
    let limiter = RateLimiter::with_config(config);
    
    // Client A should allow first 3 requests
    for _ in 0..3 {
        assert!(limiter.allow(Some("client_a")));
    }
    assert!(!limiter.allow(Some("client_a")));
    
    // Client B should have separate limit and allow first 3 requests
    for _ in 0..3 {
        assert!(limiter.allow(Some("client_b")));
    }
    assert!(!limiter.allow(Some("client_b")));
    
    // Client A should still be rejected
    assert!(!limiter.allow(Some("client_a")));
}

#[test]
fn test_rate_limiter_leaky_bucket() {
    let config = RateLimiterConfig {
        algorithm: RateLimitingAlgorithm::LeakyBucket {
            capacity: 4,
            leak_rate: 10.0,
        },
        per_client: false,
    };
    let limiter = RateLimiter::with_config(config);
    
    // Should allow first 4 requests
    for _ in 0..4 {
        assert!(limiter.allow(None));
    }
    
    // Should reject 5th request
    assert!(!limiter.allow(None));
}