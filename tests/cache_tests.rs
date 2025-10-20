use no_downtime_service::cache::{Cache, CacheConfig};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_cache_integration() {
    // Create cache with probabilistic expiration disabled to avoid test interference
    let config = CacheConfig {
        probabilistic_expiration: false,
        ..Default::default()
    };
    let cache: Cache<String, i32> = Cache::with_config(config);
    
    // Test basic operations
    cache.insert("key1".to_string(), 42, Duration::from_secs(10)).await;
    assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
    assert_eq!(cache.get(&"key2".to_string()).await, None);
    
    // Test removal
    assert_eq!(cache.remove(&"key1".to_string()).await, Some(42));
    assert_eq!(cache.get(&"key1".to_string()).await, None);
}

#[tokio::test]
async fn test_cache_get_or_compute_integration() {
    // Create cache with probabilistic expiration disabled to avoid test interference
    let config = CacheConfig {
        probabilistic_expiration: false,
        ..Default::default()
    };
    let cache: Cache<String, i32> = Cache::with_config(config);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    
    // First call should compute the value
    let result = cache.get_or_compute("key1".to_string(), || async {
        let count = counter_clone.fetch_add(1, Ordering::SeqCst);
        (count as i32, Duration::from_secs(10))
    }).await;
    
    assert_eq!(result, 0);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    
    // Second call should use cached value
    let result = cache.get_or_compute("key1".to_string(), || async {
        let count = counter_clone.fetch_add(1, Ordering::SeqCst);
        (count as i32, Duration::from_secs(10))
    }).await;
    
    assert_eq!(result, 0); // Should still be 0 from cache
    assert_eq!(counter.load(Ordering::SeqCst), 1); // Counter should still be 1
}

#[tokio::test]
async fn test_cache_expiration_integration() {
    // Create cache with probabilistic expiration disabled to avoid test interference
    let config = CacheConfig {
        probabilistic_expiration: false,
        ..Default::default()
    };
    let cache: Cache<String, i32> = Cache::with_config(config);
    
    // Insert with short TTL
    cache.insert("key1".to_string(), 42, Duration::from_millis(10)).await;
    assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
    
    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(cache.get(&"key1".to_string()).await, None);
}

#[tokio::test]
async fn test_cache_stats_integration() {
    // Create cache with probabilistic expiration disabled to avoid test interference
    let config = CacheConfig {
        probabilistic_expiration: false,
        ..Default::default()
    };
    let cache: Cache<String, i32> = Cache::with_config(config);
    
    // Initially all stats should be zero
    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.evictions, 0);
    assert_eq!(stats.current_entries, 0);
    
    // Test miss
    assert_eq!(cache.get(&"key1".to_string()).await, None);
    let stats = cache.stats();
    assert_eq!(stats.misses, 1);
    
    // Test hit
    cache.insert("key1".to_string(), 42, Duration::from_secs(10)).await;
    assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.current_entries, 1);
}

#[tokio::test]
async fn test_cache_custom_config() {
    let config = CacheConfig {
        max_entries: 5,
        default_ttl: Duration::from_secs(60),
        probabilistic_expiration: false,
        expiration_probability: 0.0,
    };
    let cache: Cache<String, i32> = Cache::with_config(config);
    
    // Test that cache works with custom config
    cache.insert("key1".to_string(), 42, Duration::from_secs(10)).await;
    assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
}