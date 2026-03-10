//! Compilation cache with LRU eviction
//!
//! Provides in-memory caching of compiled bytecode with SipHash-based collision detection.
//! Designed for <50μs cache hit latency and <10MB memory footprint for 1000 entries.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::bytecode::Bytecode;

/// LRU cache for compiled bytecode
/// Uses HashMap for O(1) lookup + collision detection via full source storage
pub struct CompilationCache {
    /// Map from source code hash to cached entry
    entries: HashMap<u64, CacheEntry>,

    /// Maximum number of entries
    capacity: usize,

    /// Monotonic timestamp for LRU tracking
    timestamp: u64,

    /// Statistics
    hits: usize,
    misses: usize,
}

/// Cached bytecode entry with full source for collision detection
struct CacheEntry {
    /// Full source code (for collision detection per PRD Risk R3)
    source: String,

    /// Compiled bytecode (Arc for cheap cloning)
    bytecode: Arc<Bytecode>,

    /// Last access timestamp
    last_access: u64,
}

impl CompilationCache {
    /// Create new cache with specified capacity
    /// Default capacity: 1000 entries
    pub fn new(capacity: usize) -> Self {
        CompilationCache {
            entries: HashMap::new(),
            capacity,
            timestamp: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Create cache with capacity from environment variable
    /// PYRUST_CACHE_SIZE controls capacity (default: 1000)
    pub fn from_env() -> Self {
        let capacity = std::env::var("PYRUST_CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);
        Self::new(capacity)
    }

    /// Get bytecode from cache
    /// Returns Some(Arc<Bytecode>) on hit, None on miss
    pub fn get(&mut self, code: &str) -> Option<Arc<Bytecode>> {
        let hash = Self::hash_code(code);

        if let Some(entry) = self.entries.get_mut(&hash) {
            // COLLISION DETECTION: verify full source matches (PRD Risk R3)
            if entry.source == code {
                self.hits += 1;

                // Update LRU timestamp (no need to update lru_order vector)
                self.timestamp += 1;
                entry.last_access = self.timestamp;

                return Some(Arc::clone(&entry.bytecode));
            } else {
                // Hash collision: different source with same hash
                // Treat as miss (rare, acceptable to recompile)
                self.misses += 1;
                return None;
            }
        }

        self.misses += 1;
        None
    }

    /// Insert compiled bytecode into cache
    /// Evicts LRU entry if capacity exceeded
    pub fn insert(&mut self, code: String, bytecode: Arc<Bytecode>) {
        // Don't insert if capacity is zero
        if self.capacity == 0 {
            return;
        }

        let hash = Self::hash_code(&code);

        // Check if already cached (update)
        if self.entries.contains_key(&hash) {
            self.entries.remove(&hash);
        }

        // Check capacity and evict if needed
        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }

        // Insert entry
        self.timestamp += 1;
        let entry = CacheEntry {
            source: code,
            bytecode,
            last_access: self.timestamp,
        };

        self.entries.insert(hash, entry);
    }

    /// Evict least recently used entry
    /// O(n) but acceptable for 1000 entry capacity
    fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        // Find oldest entry by minimum last_access timestamp
        let mut oldest_hash = 0u64;
        let mut oldest_time = u64::MAX;

        for (hash, entry) in &self.entries {
            if entry.last_access < oldest_time {
                oldest_time = entry.last_access;
                oldest_hash = *hash;
            }
        }

        self.entries.remove(&oldest_hash);
    }

    /// Hash source code using DefaultHasher (SipHash 1-3)
    /// Provides cryptographic-quality collision resistance
    fn hash_code(code: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            size: self.entries.len(),
            capacity: self.capacity,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.timestamp = 0;
        self.hits = 0;
        self.misses = 0;
    }
}

/// Cache statistics
#[derive(Debug, Clone, PartialEq)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub size: usize,
    pub capacity: usize,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expression, Program, Statement};
    use crate::compiler::compile;

    /// Helper to create a simple bytecode for testing
    fn create_bytecode(value: i64) -> Bytecode {
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Integer(value),
            }],
        };
        compile(&program).unwrap()
    }

    /// Helper to create Arc<Bytecode> for testing
    fn create_bytecode_arc(value: i64) -> Arc<Bytecode> {
        Arc::new(create_bytecode(value))
    }

    #[test]
    fn test_cache_new() {
        let cache = CompilationCache::new(100);
        let stats = cache.stats();
        assert_eq!(stats.capacity, 100);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    #[ignore] // Ignored due to env var test interference - run with --ignored --test-threads=1
    fn test_cache_from_env_default() {
        // This test must be run in isolation due to env var interference
        // Run with: cargo test test_cache_from_env -- --ignored --test-threads=1

        // Save and restore environment variable to avoid test interference
        let old_cache_value = std::env::var("PYRUST_CACHE_SIZE").ok();

        // Clear cache size var
        std::env::remove_var("PYRUST_CACHE_SIZE");

        let cache = CompilationCache::from_env();
        assert_eq!(
            cache.capacity, 1000,
            "Default capacity should be 1000 when env var is not set"
        );

        // Restore original state
        match old_cache_value {
            Some(val) => std::env::set_var("PYRUST_CACHE_SIZE", val),
            None => std::env::remove_var("PYRUST_CACHE_SIZE"),
        }
    }

    #[test]
    #[ignore] // Ignored due to env var test interference - run with --ignored --test-threads=1
    fn test_cache_from_env_custom() {
        // This test must be run in isolation due to env var interference
        // Run with: cargo test test_cache_from_env -- --ignored --test-threads=1

        // Save and restore environment variable to avoid test interference
        let old_value = std::env::var("PYRUST_CACHE_SIZE").ok();

        std::env::set_var("PYRUST_CACHE_SIZE", "500");

        let cache = CompilationCache::from_env();
        assert_eq!(cache.capacity, 500);

        // Always restore original state immediately
        match old_value {
            Some(val) => std::env::set_var("PYRUST_CACHE_SIZE", val),
            None => std::env::remove_var("PYRUST_CACHE_SIZE"),
        }
    }

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = CompilationCache::new(10);
        let code = "2 + 2";
        let bytecode = create_bytecode_arc(4);

        // First access should be a miss
        assert!(cache.get(code).is_none());
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);

        // Insert the bytecode
        cache.insert(code.to_string(), bytecode.clone());
        let stats = cache.stats();
        assert_eq!(stats.size, 1);

        // Second access should be a hit
        let result = cache.get(code);
        assert!(result.is_some());
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5); // 1 hit out of 2 accesses
    }

    #[test]
    fn test_cache_hit_rate_95_percent() {
        // AC3.1: Cache hit rate ≥95% for 100 identical requests
        let mut cache = CompilationCache::new(10);
        let code = "42";
        let bytecode = create_bytecode_arc(42);

        // First request is a miss
        cache.get(code);
        cache.insert(code.to_string(), bytecode);

        // Next 99 requests should all be hits
        for _ in 0..99 {
            assert!(cache.get(code).is_some());
        }

        let stats = cache.stats();
        assert_eq!(stats.hits, 99);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.99); // 99% hit rate
        assert!(stats.hit_rate >= 0.95); // Meets AC3.1
    }

    #[test]
    fn test_lru_eviction() {
        // AC3.4: LRU eviction works - 1001st entry evicts oldest
        let mut cache = CompilationCache::new(1000);

        // Fill cache to capacity with 1000 unique entries
        for i in 0..1000 {
            let code = format!("x = {}", i);
            let bytecode = create_bytecode_arc(i);
            cache.insert(code, bytecode);
        }

        let stats = cache.stats();
        assert_eq!(stats.size, 1000);

        // Access the first entry to make it most recently used
        let first_code = "x = 0";
        assert!(cache.get(first_code).is_some());

        // Insert 1001st entry - should evict "x = 1" (oldest non-accessed)
        let new_code = "x = 1000";
        let new_bytecode = create_bytecode_arc(1000);
        cache.insert(new_code.to_string(), new_bytecode);

        let stats = cache.stats();
        assert_eq!(stats.size, 1000); // Still at capacity

        // First entry should still be in cache (was recently accessed)
        assert!(cache.get(first_code).is_some());

        // Second entry should have been evicted
        let second_code = "x = 1";
        assert!(cache.get(second_code).is_none());

        // New entry should be in cache
        assert!(cache.get(new_code).is_some());
    }

    #[test]
    fn test_collision_detection() {
        // AC3.6: Cache invalidation - different code produces different results
        let mut cache = CompilationCache::new(10);

        let code1 = "2 + 2";
        let code2 = "3 + 3";
        let bytecode1 = create_bytecode_arc(4);
        let bytecode2 = create_bytecode_arc(6);

        // Insert first code
        cache.insert(code1.to_string(), bytecode1);

        // Get first code - should succeed
        let result1 = cache.get(code1);
        assert!(result1.is_some());

        // Get different code - should be a miss
        let result2 = cache.get(code2);
        assert!(result2.is_none());

        // Insert second code
        cache.insert(code2.to_string(), bytecode2);

        // Both should now be accessible
        assert!(cache.get(code1).is_some());
        assert!(cache.get(code2).is_some());

        // Verify they're different bytecode objects
        let bc1 = cache.get(code1).unwrap();
        let bc2 = cache.get(code2).unwrap();

        // They should have different constants
        assert_ne!(bc1.constants, bc2.constants);
    }

    #[test]
    fn test_capacity_limit() {
        let mut cache = CompilationCache::new(5);

        // Insert 10 entries into a cache with capacity 5
        for i in 0..10 {
            let code = format!("x = {}", i);
            let bytecode = create_bytecode_arc(i);
            cache.insert(code, bytecode);
        }

        // Cache should never exceed capacity
        let stats = cache.stats();
        assert_eq!(stats.size, 5);
        assert!(stats.size <= stats.capacity);
    }

    #[test]
    fn test_clear() {
        let mut cache = CompilationCache::new(10);

        // Add some entries
        for i in 0..5 {
            let code = format!("x = {}", i);
            let bytecode = create_bytecode_arc(i);
            cache.insert(code, bytecode);
        }

        // Access some entries to generate hits
        cache.get("x = 0");
        cache.get("x = 1");

        // Verify cache has content
        let stats = cache.stats();
        assert_eq!(stats.size, 5);
        assert!(stats.hits > 0);

        // Clear cache
        cache.clear();

        // Verify everything is reset
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
        assert_eq!(cache.timestamp, 0);
    }

    #[test]
    fn test_empty_cache_stats() {
        let cache = CompilationCache::new(10);
        let stats = cache.stats();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_single_entry_cache() {
        let mut cache = CompilationCache::new(1);
        let code1 = "42";
        let code2 = "43";
        let bytecode1 = create_bytecode_arc(42);
        let bytecode2 = create_bytecode_arc(43);

        // Insert first entry
        cache.insert(code1.to_string(), bytecode1);
        assert_eq!(cache.stats().size, 1);
        assert!(cache.get(code1).is_some());

        // Insert second entry - should evict first
        cache.insert(code2.to_string(), bytecode2);
        assert_eq!(cache.stats().size, 1);
        assert!(cache.get(code2).is_some());
        assert!(cache.get(code1).is_none());
    }

    #[test]
    fn test_zero_capacity() {
        let mut cache = CompilationCache::new(0);
        let code = "42";
        let bytecode = create_bytecode_arc(42);

        // Attempt to insert - should not crash
        cache.insert(code.to_string(), bytecode);

        // Cache should remain empty
        assert_eq!(cache.stats().size, 0);
        assert!(cache.get(code).is_none());
    }

    #[test]
    fn test_update_existing_entry() {
        let mut cache = CompilationCache::new(10);
        let code = "2 + 2";
        let bytecode1 = create_bytecode_arc(4);
        let bytecode2 = create_bytecode_arc(5); // Different bytecode for same source

        // Insert first version
        cache.insert(code.to_string(), bytecode1);
        assert_eq!(cache.stats().size, 1);

        // Insert second version - should update, not add
        cache.insert(code.to_string(), bytecode2);
        assert_eq!(cache.stats().size, 1); // Still 1 entry

        // Verify updated bytecode is returned
        let result = cache.get(code).unwrap();
        assert_eq!(result.constants[0], 5);
    }

    #[test]
    fn test_hash_consistency() {
        let code = "2 + 2";
        let hash1 = CompilationCache::hash_code(code);
        let hash2 = CompilationCache::hash_code(code);

        // Same code should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_uniqueness() {
        let code1 = "2 + 2";
        let code2 = "3 + 3";
        let hash1 = CompilationCache::hash_code(code1);
        let hash2 = CompilationCache::hash_code(code2);

        // Different code should produce different hash (highly likely)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_arc_cloning() {
        let mut cache = CompilationCache::new(10);
        let code = "42";
        let bytecode = create_bytecode_arc(42);

        cache.insert(code.to_string(), bytecode);

        // Get the same bytecode multiple times
        let bc1 = cache.get(code).unwrap();
        let bc2 = cache.get(code).unwrap();

        // Arc cloning should give us references to the same data
        assert!(Arc::ptr_eq(&bc1, &bc2));
    }

    #[test]
    fn test_lru_updates_on_access() {
        let mut cache = CompilationCache::new(3);

        // Insert 3 entries
        for i in 0..3 {
            let code = format!("x = {}", i);
            let bytecode = create_bytecode_arc(i);
            cache.insert(code, bytecode);
        }

        // Access the first entry to make it recent
        cache.get("x = 0");

        // Insert a new entry - should evict "x = 1" (oldest)
        cache.insert("x = 3".to_string(), create_bytecode_arc(3));

        // "x = 0" should still be in cache (was accessed)
        assert!(cache.get("x = 0").is_some());

        // "x = 1" should have been evicted
        assert!(cache.get("x = 1").is_none());

        // "x = 2" and "x = 3" should be in cache
        assert!(cache.get("x = 2").is_some());
        assert!(cache.get("x = 3").is_some());
    }

    #[test]
    fn test_memory_footprint_estimate() {
        // AC3.5: Memory usage ≤10MB for 1000 cached scripts
        // This test provides a rough estimate
        let mut cache = CompilationCache::new(1000);

        // Create 1000 entries with realistic-sized code
        for i in 0..1000 {
            // ~50 character source code (realistic average)
            let code = format!("x = {}\ny = {}\nz = x + y\nprint(z)", i, i * 2);
            let bytecode = create_bytecode_arc(i);
            cache.insert(code, bytecode);
        }

        let stats = cache.stats();
        assert_eq!(stats.size, 1000);

        // Memory estimate:
        // - HashMap overhead: ~48 bytes per entry
        // - CacheEntry: ~50 bytes source + ~200 bytes bytecode (avg) + 8 bytes timestamp
        // - Total per entry: ~306 bytes
        // - 1000 entries: ~306KB
        // - Plus HashMap/Vec overhead: ~1MB total
        // Well under 10MB limit

        // This test mainly verifies the cache can hold 1000 entries
        // Actual memory profiling would be done with tools like dhat
        assert!(stats.size == 1000);
    }

    #[test]
    fn test_stats_clone() {
        let cache = CompilationCache::new(10);
        let stats1 = cache.stats();
        let stats2 = stats1.clone();

        assert_eq!(stats1, stats2);
    }

    #[test]
    fn test_different_sources_same_result() {
        // Ensure that even if two different sources produce the same result value,
        // they are cached separately
        let mut cache = CompilationCache::new(10);

        let code1 = "4";
        let code2 = "2 + 2";
        let bytecode1 = create_bytecode_arc(4);
        let bytecode2 = create_bytecode_arc(4);

        cache.insert(code1.to_string(), bytecode1);
        cache.insert(code2.to_string(), bytecode2);

        // Both should be in cache
        assert!(cache.get(code1).is_some());
        assert!(cache.get(code2).is_some());

        // Cache should have 2 entries
        assert_eq!(cache.stats().size, 2);
    }

    // EDGE CASE TESTS

    #[test]
    fn test_empty_string_caching() {
        // Edge case: empty source code
        let mut cache = CompilationCache::new(10);
        let code = "";
        let bytecode = create_bytecode_arc(0);

        // Should be able to cache empty string
        cache.insert(code.to_string(), bytecode);
        assert_eq!(cache.stats().size, 1);

        // Should be able to retrieve empty string
        let result = cache.get(code);
        assert!(result.is_some());
    }

    #[test]
    fn test_whitespace_only_caching() {
        // Edge case: whitespace-only source code
        let mut cache = CompilationCache::new(10);
        let code1 = "   ";
        let code2 = "\t\t";
        let code3 = "\n\n";
        let bytecode = create_bytecode_arc(0);

        // Different whitespace should be cached separately
        cache.insert(code1.to_string(), bytecode.clone());
        cache.insert(code2.to_string(), bytecode.clone());
        cache.insert(code3.to_string(), bytecode);

        assert_eq!(cache.stats().size, 3);
        assert!(cache.get(code1).is_some());
        assert!(cache.get(code2).is_some());
        assert!(cache.get(code3).is_some());
    }

    #[test]
    fn test_very_long_source_code() {
        // Edge case: very long source code
        let mut cache = CompilationCache::new(10);

        // Create a 10KB source string
        let code = "x = 42\n".repeat(1000);
        let bytecode = create_bytecode_arc(42);

        cache.insert(code.clone(), bytecode);
        assert_eq!(cache.stats().size, 1);

        // Should be able to retrieve long source
        let result = cache.get(&code);
        assert!(result.is_some());
    }

    #[test]
    fn test_special_characters_in_source() {
        // Edge case: source with special characters
        let mut cache = CompilationCache::new(10);
        let code = "# Comment with émojis 🚀\nx = 42";
        let bytecode = create_bytecode_arc(42);

        cache.insert(code.to_string(), bytecode);
        assert_eq!(cache.stats().size, 1);

        // Should be able to retrieve source with special chars
        let result = cache.get(code);
        assert!(result.is_some());
    }

    #[test]
    fn test_similar_sources_different_hashes() {
        // Edge case: very similar sources should have different hashes
        let mut cache = CompilationCache::new(10);

        let code1 = "x = 1";
        let code2 = "x = 2";
        let bytecode1 = create_bytecode_arc(1);
        let bytecode2 = create_bytecode_arc(2);

        cache.insert(code1.to_string(), bytecode1);
        cache.insert(code2.to_string(), bytecode2);

        // Should have 2 entries
        assert_eq!(cache.stats().size, 2);

        // Should return different bytecode
        let bc1 = cache.get(code1).unwrap();
        let bc2 = cache.get(code2).unwrap();
        assert_ne!(bc1.constants[0], bc2.constants[0]);
    }

    #[test]
    fn test_repeated_inserts_same_source() {
        // Edge case: repeatedly inserting the same source
        let mut cache = CompilationCache::new(10);
        let code = "42";

        // Insert same source 5 times
        for i in 0..5 {
            let bytecode = create_bytecode_arc(i);
            cache.insert(code.to_string(), bytecode);
        }

        // Should only have 1 entry (updated each time)
        assert_eq!(cache.stats().size, 1);

        // Should have the latest value
        let result = cache.get(code).unwrap();
        assert_eq!(result.constants[0], 4);
    }

    #[test]
    fn test_cache_miss_does_not_affect_size() {
        // Edge case: cache misses should not affect cache size
        let mut cache = CompilationCache::new(10);
        let code = "42";
        let bytecode = create_bytecode_arc(42);

        cache.insert(code.to_string(), bytecode);
        assert_eq!(cache.stats().size, 1);

        // Multiple misses
        cache.get("43");
        cache.get("44");
        cache.get("45");

        // Size should still be 1
        assert_eq!(cache.stats().size, 1);
    }

    #[test]
    fn test_eviction_with_repeated_access() {
        // Edge case: frequently accessed entries should survive eviction longer
        let mut cache = CompilationCache::new(3);

        // Insert 3 entries
        cache.insert("x = 1".to_string(), create_bytecode_arc(1));
        cache.insert("x = 2".to_string(), create_bytecode_arc(2));
        cache.insert("x = 3".to_string(), create_bytecode_arc(3));

        // Access first entry multiple times
        for _ in 0..5 {
            cache.get("x = 1");
        }

        // Insert new entry - should evict "x = 2" (least recently used)
        cache.insert("x = 4".to_string(), create_bytecode_arc(4));

        // First entry should still be present (frequently accessed)
        assert!(cache.get("x = 1").is_some());

        // Second entry should be evicted
        assert!(cache.get("x = 2").is_none());

        // Third and fourth should be present
        assert!(cache.get("x = 3").is_some());
        assert!(cache.get("x = 4").is_some());
    }

    #[test]
    fn test_hit_rate_calculation_edge_cases() {
        // Edge case: hit rate with no accesses
        let cache = CompilationCache::new(10);
        assert_eq!(cache.stats().hit_rate, 0.0);

        // Edge case: 100% hit rate
        let mut cache = CompilationCache::new(10);
        cache.insert("42".to_string(), create_bytecode_arc(42));
        for _ in 0..10 {
            cache.get("42");
        }
        assert_eq!(cache.stats().hit_rate, 1.0);

        // Edge case: 0% hit rate
        let mut cache = CompilationCache::new(10);
        for i in 0..10 {
            cache.get(&format!("x = {}", i));
        }
        assert_eq!(cache.stats().hit_rate, 0.0);
    }

    #[test]
    fn test_max_capacity_edge_case() {
        // Edge case: large capacity value
        let cache = CompilationCache::new(100000);
        assert_eq!(cache.capacity, 100000);
        assert_eq!(cache.stats().size, 0);
    }

    #[test]
    fn test_collision_detection_false_positive_prevention() {
        // Edge case: ensure collision detection prevents false positives
        // Even if we get a hash collision, the full source comparison should catch it
        let mut cache = CompilationCache::new(10);

        let code1 = "abc";
        let code2 = "xyz";
        let bytecode1 = create_bytecode_arc(1);
        let bytecode2 = create_bytecode_arc(2);

        cache.insert(code1.to_string(), bytecode1);

        // Manually force a collision scenario by testing with different code
        let result = cache.get(code2);

        // Should be a miss (different source)
        assert!(result.is_none());

        // Insert the second code
        cache.insert(code2.to_string(), bytecode2);

        // Both should be retrievable with correct values
        let bc1 = cache.get(code1).unwrap();
        let bc2 = cache.get(code2).unwrap();
        assert_eq!(bc1.constants[0], 1);
        assert_eq!(bc2.constants[0], 2);
    }

    #[test]
    fn test_interleaved_operations() {
        // Edge case: interleaved get/insert operations
        let mut cache = CompilationCache::new(5);

        // Insert, get, insert, get pattern
        cache.insert("a".to_string(), create_bytecode_arc(1));
        assert!(cache.get("a").is_some());

        cache.insert("b".to_string(), create_bytecode_arc(2));
        assert!(cache.get("b").is_some());
        assert!(cache.get("a").is_some());

        cache.insert("c".to_string(), create_bytecode_arc(3));
        assert!(cache.get("c").is_some());

        let stats = cache.stats();
        assert_eq!(stats.size, 3);
        assert_eq!(stats.hits, 4);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_unicode_source_code() {
        // Edge case: Unicode in source code
        let mut cache = CompilationCache::new(10);
        let code1 = "变量 = 42"; // Chinese characters
        let code2 = "переменная = 42"; // Cyrillic characters
        let code3 = "変数 = 42"; // Japanese characters

        cache.insert(code1.to_string(), create_bytecode_arc(1));
        cache.insert(code2.to_string(), create_bytecode_arc(2));
        cache.insert(code3.to_string(), create_bytecode_arc(3));

        assert_eq!(cache.stats().size, 3);
        assert!(cache.get(code1).is_some());
        assert!(cache.get(code2).is_some());
        assert!(cache.get(code3).is_some());
    }

    #[test]
    fn test_concurrent_get_and_insert() {
        // Edge case: interleaving gets and inserts
        let mut cache = CompilationCache::new(5);

        // Insert some entries
        for i in 0..3 {
            cache.insert(format!("x = {}", i), create_bytecode_arc(i));
        }

        // Interleave gets and inserts
        assert!(cache.get("x = 0").is_some());
        cache.insert("x = 3".to_string(), create_bytecode_arc(3));
        assert!(cache.get("x = 1").is_some());
        cache.insert("x = 4".to_string(), create_bytecode_arc(4));
        assert!(cache.get("x = 2").is_some());

        // All should be accessible
        assert_eq!(cache.stats().size, 5);
    }

    #[test]
    fn test_get_updates_lru_order() {
        // Verify that get() actually updates LRU order
        let mut cache = CompilationCache::new(3);

        // Insert 3 entries
        cache.insert("a".to_string(), create_bytecode_arc(1));
        cache.insert("b".to_string(), create_bytecode_arc(2));
        cache.insert("c".to_string(), create_bytecode_arc(3));

        // Access 'a' to make it most recent
        let _ = cache.get("a");

        // Insert new entry - should evict 'b' (least recently used)
        cache.insert("d".to_string(), create_bytecode_arc(4));

        // 'a' should still be present
        assert!(cache.get("a").is_some());

        // 'b' should have been evicted
        assert!(cache.get("b").is_none());

        // 'c' and 'd' should be present
        assert!(cache.get("c").is_some());
        assert!(cache.get("d").is_some());
    }

    #[test]
    fn test_collision_with_same_hash_different_source() {
        // Test behavior when two different strings theoretically have the same hash
        // (we can't easily force a hash collision, but we test the collision detection logic)
        let mut cache = CompilationCache::new(10);

        let code1 = "test_code_1";
        let code2 = "test_code_2";
        let bytecode1 = create_bytecode_arc(1);
        let bytecode2 = create_bytecode_arc(2);

        cache.insert(code1.to_string(), bytecode1);
        cache.insert(code2.to_string(), bytecode2);

        // Both should be retrievable with correct values
        let bc1 = cache.get(code1).unwrap();
        let bc2 = cache.get(code2).unwrap();

        assert_eq!(bc1.constants[0], 1);
        assert_eq!(bc2.constants[0], 2);

        // Should have 2 separate entries
        assert_eq!(cache.stats().size, 2);
    }

    #[test]
    fn test_cache_stress_test() {
        // Stress test: many operations
        let mut cache = CompilationCache::new(100);

        // Insert 200 entries (will cause evictions)
        for i in 0..200 {
            let code = format!("x = {}", i);
            cache.insert(code, create_bytecode_arc(i));
        }

        // Cache should be at capacity
        assert_eq!(cache.stats().size, 100);

        // Access some entries
        for i in 100..200 {
            let code = format!("x = {}", i);
            assert!(cache.get(&code).is_some());
        }

        // Verify hit rate is reasonable
        let stats = cache.stats();
        assert!(stats.hits > 0);
    }

    #[test]
    fn test_insert_after_clear() {
        // Edge case: insert after clear
        let mut cache = CompilationCache::new(10);

        // Add entries
        for i in 0..5 {
            cache.insert(format!("x = {}", i), create_bytecode_arc(i));
        }

        // Clear
        cache.clear();

        // Insert new entry
        cache.insert("y = 42".to_string(), create_bytecode_arc(42));

        assert_eq!(cache.stats().size, 1);
        assert!(cache.get("y = 42").is_some());
    }

    #[test]
    fn test_exact_capacity_boundary() {
        // Test behavior exactly at capacity boundary
        let mut cache = CompilationCache::new(10);

        // Fill to exactly capacity
        for i in 0..10 {
            cache.insert(format!("x = {}", i), create_bytecode_arc(i));
        }

        assert_eq!(cache.stats().size, 10);

        // All entries should be accessible
        for i in 0..10 {
            let code = format!("x = {}", i);
            assert!(cache.get(&code).is_some());
        }

        // Add one more - should evict oldest
        cache.insert("x = 10".to_string(), create_bytecode_arc(10));

        assert_eq!(cache.stats().size, 10);
        assert!(cache.get("x = 0").is_none()); // First entry should be evicted
        assert!(cache.get("x = 10").is_some()); // New entry should be present
    }

    #[test]
    fn test_hit_miss_statistics_accuracy() {
        // Verify hit/miss statistics are accurate
        let mut cache = CompilationCache::new(10);

        // Insert 3 entries
        cache.insert("a".to_string(), create_bytecode_arc(1));
        cache.insert("b".to_string(), create_bytecode_arc(2));
        cache.insert("c".to_string(), create_bytecode_arc(3));

        // 5 hits
        for _ in 0..5 {
            cache.get("a");
        }

        // 3 misses
        for i in 0..3 {
            cache.get(&format!("missing_{}", i));
        }

        let stats = cache.stats();
        assert_eq!(stats.hits, 5);
        assert_eq!(stats.misses, 3);
        assert!((stats.hit_rate - 0.625).abs() < 0.001); // 5/8 = 0.625
    }
}
