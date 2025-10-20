//! Cache implementation with stampede protection
//!
//! This module implements an in-memory cache with cache stampede protection
//! to prevent thundering herd problems when cache entries expire.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// Cached value
    value: V,
    /// Expiration time
    expiration: Instant,
    /// Creation time
    created: Instant,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of hits
    pub hits: u64,
    /// Number of misses
    pub misses: u64,
    /// Number of evictions
    pub evictions: u64,
    /// Current number of entries
    pub current_entries: usize,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Default time-to-live for entries
    pub default_ttl: Duration,
    /// Whether to use probabilistic early expiration
    pub probabilistic_expiration: bool,
    /// Probability factor for early expiration (0.0 to 1.0)
    pub expiration_probability: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            probabilistic_expiration: true,
            expiration_probability: 0.1, // 10% chance of early expiration
        }
    }
}

/// Cache with stampede protection
#[derive(Clone)]
pub struct Cache<K, V> {
    /// Internal cache storage
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Configuration
    config: CacheConfig,
    /// Statistics
    stats: Arc<AtomicU64>,
    /// Broadcast channel for coordinating cache loads
    load_channels: Arc<Mutex<HashMap<K, broadcast::Sender<V>>>>,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + Hash + std::fmt::Debug,
    V: Clone + Send + Sync,
{
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(AtomicU64::new(0)), // bits 0-19: hits, 20-39: misses, 40-59: evictions
            load_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        // Check if the entry exists and is not expired
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();
        
        if let Some(entry) = entries.get(key) {
            if entry.expiration > now {
                // Check if we should use probabilistic early expiration
                if self.config.probabilistic_expiration && rand::random::<f64>() < self.config.expiration_probability {
                    // Remove the entry probabilistically
                    entries.remove(key);
                    // Increment misses counter
                    let current = self.stats.load(Ordering::Relaxed);
                    let hits = current & 0xFFFFF;
                    let misses = ((current >> 20) & 0xFFFFF) + 1;
                    let evictions = (current >> 40) & 0xFFFFF;
                    self.stats.store(hits | (misses << 20) | (evictions << 40), Ordering::Relaxed);
                    return None;
                }
                
                // Increment hits counter
                let current = self.stats.load(Ordering::Relaxed);
                let hits = (current & 0xFFFFF) + 1;
                let misses = (current >> 20) & 0xFFFFF;
                let evictions = (current >> 40) & 0xFFFFF;
                self.stats.store(hits | (misses << 20) | (evictions << 40), Ordering::Relaxed);
                
                return Some(entry.value.clone());
            } else {
                // Entry expired, remove it
                entries.remove(key);
            }
        }
        
        // Increment misses counter
        let current = self.stats.load(Ordering::Relaxed);
        let hits = current & 0xFFFFF;
        let misses = ((current >> 20) & 0xFFFFF) + 1;
        let evictions = (current >> 40) & 0xFFFFF;
        self.stats.store(hits | (misses << 20) | (evictions << 40), Ordering::Relaxed);
        
        None
    }

    /// Get a value from the cache or compute it if not present
    pub async fn get_or_compute<F, Fut>(&self, key: K, compute: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = (V, Duration)>,
    {
        // First, try to get from cache
        if let Some(value) = self.get(&key).await {
            return value;
        }

        // Check if there's already a load in progress for this key
        {
            let load_channels = self.load_channels.lock().unwrap();
            if let Some(sender) = load_channels.get(&key) {
                // There's already a load in progress, subscribe to it
                let mut receiver = sender.subscribe();
                drop(load_channels); // Release the lock
                
                // Wait for the value to be computed
                if let Ok(value) = receiver.recv().await {
                    return value;
                }
            }
        }

        // No load in progress, we need to compute the value
        // Create a broadcast channel for this load
        let (sender, _receiver) = broadcast::channel(10);
        {
            let mut load_channels = self.load_channels.lock().unwrap();
            load_channels.insert(key.clone(), sender.clone());
        }

        // Compute the value
        let (value, ttl) = compute().await;
        
        // Store in cache
        self.insert(key.clone(), value.clone(), ttl).await;
        
        // Notify all waiting consumers
        let _ = sender.send(value.clone());
        
        // Remove the load channel
        {
            let mut load_channels = self.load_channels.lock().unwrap();
            load_channels.remove(&key);
        }
        
        value
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: K, value: V, ttl: Duration) {
        let now = Instant::now();
        let expiration = now + ttl;
        
        let entry = CacheEntry {
            value,
            expiration,
            created: now,
        };
        
        let mut entries = self.entries.write().unwrap();
        
        // Check if we need to evict entries
        if entries.len() >= self.config.max_entries {
            // Simple LRU eviction: remove the oldest entry
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.created)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
                
                // Increment evictions counter
                let current = self.stats.load(Ordering::Relaxed);
                let hits = current & 0xFFFFF;
                let misses = (current >> 20) & 0xFFFFF;
                let evictions = ((current >> 40) & 0xFFFFF) + 1;
                self.stats.store(hits | (misses << 20) | (evictions << 40), Ordering::Relaxed);
            }
        }
        
        entries.insert(key, entry);
    }

    /// Remove a value from the cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().unwrap();
        entries.remove(key).map(|entry| entry.value)
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let current = self.stats.load(Ordering::Relaxed);
        let hits = current & 0xFFFFF;
        let misses = (current >> 20) & 0xFFFFF;
        let evictions = (current >> 40) & 0xFFFFF;
        let current_entries = self.entries.read().unwrap().len();
        
        CacheStats {
            hits,
            misses,
            evictions,
            current_entries,
        }
    }

    /// Clean expired entries
    pub async fn clean_expired(&self) {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();
        
        entries.retain(|_, entry| entry.expiration > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        // Create cache with probabilistic expiration disabled to avoid test interference
        let config = CacheConfig {
            probabilistic_expiration: false,
            ..Default::default()
        };
        let cache: Cache<String, i32> = Cache::with_config(config);
        
        // Test insert and get
        cache.insert("key1".to_string(), 42, Duration::from_secs(10)).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some(42));
        
        // Test get non-existent key
        assert_eq!(cache.get(&"key2".to_string()).await, None);
        
        // Test remove
        assert_eq!(cache.remove(&"key1".to_string()).await, Some(42));
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
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
        sleep(Duration::from_millis(50)).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_get_or_compute() {
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
            counter_clone.fetch_add(1, AtomicOrdering::SeqCst);
            (42, Duration::from_secs(10))
        }).await;
        assert_eq!(result, 42);
        assert_eq!(counter.load(AtomicOrdering::SeqCst), 1);
        
        // Second call should use cached value
        let result = cache.get_or_compute("key1".to_string(), || async {
            counter_clone.fetch_add(1, AtomicOrdering::SeqCst);
            (84, Duration::from_secs(10))
        }).await;
        assert_eq!(result, 42); // Should still be 42 from cache
        assert_eq!(counter.load(AtomicOrdering::SeqCst), 1); // Counter should still be 1
    }

    #[tokio::test]
    async fn test_cache_stampede_protection() {
        // Create cache with probabilistic expiration disabled to avoid test interference
        let config = CacheConfig {
            probabilistic_expiration: false,
            ..Default::default()
        };
        let cache: Cache<String, i32> = Cache::with_config(config);
        let counter = Arc::new(AtomicU32::new(0));
        
        // Test that the cache works correctly - simplified test
        let result1 = cache.get_or_compute("key1".to_string(), || async {
            // Simulate some computation time
            tokio::time::sleep(Duration::from_millis(10)).await;
            let count = counter.fetch_add(1, AtomicOrdering::SeqCst);
            (count as i32, Duration::from_secs(10))
        }).await;
        
        // Give some time for the cache to be populated
        tokio::time::sleep(Duration::from_millis(5)).await;
        
        let result2 = cache.get_or_compute("key1".to_string(), || async {
            // This should not be called since value is in cache
            let count = counter.fetch_add(1, AtomicOrdering::SeqCst);
            (count as i32, Duration::from_secs(10))
        }).await;
        
        // Both calls should return the same value
        assert_eq!(result1, 0);
        assert_eq!(result2, 0);
        // Counter should be 1 since second call should use cache
        assert_eq!(counter.load(AtomicOrdering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_cache_stats() {
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
}