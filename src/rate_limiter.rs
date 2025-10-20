//! Rate limiter implementation for resilience patterns
//!
//! This module implements rate limiting to prevent service overload
//! and provide backpressure mechanisms.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Rate limiting algorithm
#[derive(Debug, Clone)]
pub enum RateLimitingAlgorithm {
    /// Token bucket algorithm
    TokenBucket {
        /// Maximum number of tokens in the bucket
        max_tokens: u32,
        /// Tokens added per second
        tokens_per_second: f64,
    },
    /// Leaky bucket algorithm
    LeakyBucket {
        /// Maximum capacity of the bucket
        capacity: u32,
        /// Leak rate (requests per second)
        leak_rate: f64,
    },
}

/// Configuration for the rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Algorithm to use for rate limiting
    pub algorithm: RateLimitingAlgorithm,
    /// Whether to limit globally or per client
    pub per_client: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            algorithm: RateLimitingAlgorithm::TokenBucket {
                max_tokens: 100,
                tokens_per_second: 10.0,
            },
            per_client: false,
        }
    }
}

/// Token bucket implementation
#[derive(Debug)]
struct TokenBucket {
    /// Maximum number of tokens
    max_tokens: u32,
    /// Current number of tokens
    tokens: f64,
    /// Tokens added per second
    tokens_per_second: f64,
    /// Last time tokens were updated
    last_update: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(max_tokens: u32, tokens_per_second: f64) -> Self {
        Self {
            max_tokens,
            tokens: max_tokens as f64,
            tokens_per_second,
            last_update: Instant::now(),
        }
    }

    /// Check if a request is allowed and consume a token if so
    fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        // Add tokens based on elapsed time
        self.tokens = (self.tokens + elapsed * self.tokens_per_second).min(self.max_tokens as f64);
        self.last_update = now;
        
        // Check if we have a token to consume
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Leaky bucket implementation
#[derive(Debug)]
struct LeakyBucket {
    /// Maximum capacity
    capacity: u32,
    /// Current number of requests in the bucket
    requests: u32,
    /// Leak rate (requests per second)
    leak_rate: f64,
    /// Last time the bucket was updated
    last_update: Instant,
}

impl LeakyBucket {
    /// Create a new leaky bucket
    fn new(capacity: u32, leak_rate: f64) -> Self {
        Self {
            capacity,
            requests: 0,
            leak_rate,
            last_update: Instant::now(),
        }
    }

    /// Check if a request is allowed and add it to the bucket if so
    fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        // Remove requests based on leak rate
        let leaked = (elapsed * self.leak_rate) as u32;
        self.requests = self.requests.saturating_sub(leaked);
        self.last_update = now;
        
        // Check if we can add a new request
        if self.requests < self.capacity {
            self.requests += 1;
            true
        } else {
            false
        }
    }
}

/// Rate limiter implementation
pub struct RateLimiter {
    /// Configuration
    config: RateLimiterConfig,
    /// Token buckets for token bucket algorithm
    token_buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    /// Leaky buckets for leaky bucket algorithm
    leaky_buckets: Arc<Mutex<HashMap<String, LeakyBucket>>>,
    /// Global bucket for non-per-client limiting
    global_bucket: Arc<Mutex<Option<Box<dyn RateLimitingBucket>>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new() -> Self {
        Self::with_config(RateLimiterConfig::default())
    }

    /// Create a new rate limiter with custom configuration
    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self {
            config,
            token_buckets: Arc::new(Mutex::new(HashMap::new())),
            leaky_buckets: Arc::new(Mutex::new(HashMap::new())),
            global_bucket: Arc::new(Mutex::new(None)),
        }
    }

    /// Check if a request is allowed
    pub fn allow(&self, client_id: Option<&str>) -> bool {
        let client_id = client_id.unwrap_or("global");
        
        match &self.config.algorithm {
            RateLimitingAlgorithm::TokenBucket { max_tokens, tokens_per_second } => {
                if self.config.per_client {
                    let mut buckets = self.token_buckets.lock().unwrap();
                    let bucket = buckets.entry(client_id.to_string()).or_insert_with(|| {
                        TokenBucket::new(*max_tokens, *tokens_per_second)
                    });
                    bucket.allow()
                } else {
                    let mut global_bucket = self.global_bucket.lock().unwrap();
                    if global_bucket.is_none() {
                        *global_bucket = Some(Box::new(TokenBucket::new(*max_tokens, *tokens_per_second)));
                    }
                    if let Some(bucket) = global_bucket.as_mut() {
                        bucket.as_mut().allow()
                    } else {
                        true // Should not happen
                    }
                }
            }
            RateLimitingAlgorithm::LeakyBucket { capacity, leak_rate } => {
                if self.config.per_client {
                    let mut buckets = self.leaky_buckets.lock().unwrap();
                    let bucket = buckets.entry(client_id.to_string()).or_insert_with(|| {
                        LeakyBucket::new(*capacity, *leak_rate)
                    });
                    bucket.allow()
                } else {
                    let mut global_bucket = self.global_bucket.lock().unwrap();
                    if global_bucket.is_none() {
                        *global_bucket = Some(Box::new(LeakyBucket::new(*capacity, *leak_rate)));
                    }
                    if let Some(bucket) = global_bucket.as_mut() {
                        bucket.as_mut().allow()
                    } else {
                        true // Should not happen
                    }
                }
            }
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }
}

/// Trait for rate limiting buckets
trait RateLimitingBucket {
    /// Check if a request is allowed
    fn allow(&mut self) -> bool;
}

impl RateLimitingBucket for TokenBucket {
    fn allow(&mut self) -> bool {
        TokenBucket::allow(self)
    }
}

impl RateLimitingBucket for LeakyBucket {
    fn allow(&mut self) -> bool {
        LeakyBucket::allow(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_token_bucket_allow() {
        let mut bucket = TokenBucket::new(10, 5.0); // 10 tokens, 5 per second
        
        // Should allow first request
        assert!(bucket.allow());
        
        // Should allow more requests until tokens are depleted
        for _ in 0..9 {
            assert!(bucket.allow());
        }
        
        // Should reject when tokens are depleted
        assert!(!bucket.allow());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(5, 10.0); // 5 tokens, 10 per second
        
        // Deplete tokens
        for _ in 0..5 {
            assert!(bucket.allow());
        }
        assert!(!bucket.allow());
        
        // Wait for refill
        thread::sleep(Duration::from_millis(200)); // 0.2 seconds should refill ~2 tokens
        
        // Should allow some requests now
        assert!(bucket.allow());
        assert!(bucket.allow());
    }

    #[test]
    fn test_leaky_bucket_allow() {
        let mut bucket = LeakyBucket::new(5, 10.0); // capacity 5, leak 10 per second
        
        // Should allow requests until capacity is reached
        for _ in 0..5 {
            assert!(bucket.allow());
        }
        
        // Should reject when capacity is reached
        assert!(!bucket.allow());
    }

    #[test]
    fn test_rate_limiter_token_bucket() {
        let config = RateLimiterConfig {
            algorithm: RateLimitingAlgorithm::TokenBucket {
                max_tokens: 3,
                tokens_per_second: 10.0,
            },
            per_client: false,
        };
        let limiter = RateLimiter::with_config(config);
        
        // Should allow first requests
        assert!(limiter.allow(None));
        assert!(limiter.allow(None));
        assert!(limiter.allow(None));
        
        // Should reject when tokens are depleted
        assert!(!limiter.allow(None));
    }

    #[test]
    fn test_rate_limiter_per_client() {
        let config = RateLimiterConfig {
            algorithm: RateLimitingAlgorithm::TokenBucket {
                max_tokens: 2,
                tokens_per_second: 10.0,
            },
            per_client: true,
        };
        let limiter = RateLimiter::with_config(config);
        
        // Client A should allow first requests
        assert!(limiter.allow(Some("client_a")));
        assert!(limiter.allow(Some("client_a")));
        assert!(!limiter.allow(Some("client_a")));
        
        // Client B should have separate limit
        assert!(limiter.allow(Some("client_b")));
        assert!(limiter.allow(Some("client_b")));
        assert!(!limiter.allow(Some("client_b")));
    }
}