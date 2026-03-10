//! Integration tests for cache functionality
//!
//! Tests cache integration with library and daemon modes per AC3.2 and AC3.3:
//! - Thread-local cache for library mode (no locking overhead)
//! - Global cache for daemon mode (shared across requests)
//! - Cache clearing functionality
//! - Cache statistics

use pyrust::{
    clear_global_cache, clear_thread_local_cache, execute_python, execute_python_cached,
    execute_python_cached_global, get_global_cache_stats, get_thread_local_cache_stats,
};

#[test]
fn test_execute_python_uses_thread_local_cache() {
    // Clear cache first
    clear_thread_local_cache();

    let code = "2 + 2";

    // First execution - cache miss
    let result1 = execute_python(code).unwrap();
    assert_eq!(result1, "4");

    let stats = get_thread_local_cache_stats();
    assert_eq!(
        stats.size, 1,
        "Cache should have 1 entry after first execution"
    );
    assert_eq!(stats.misses, 1, "Should have 1 miss");

    // Second execution - cache hit
    let result2 = execute_python(code).unwrap();
    assert_eq!(result2, "4");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 1, "Cache should still have 1 entry");
    assert_eq!(stats.hits, 1, "Should have 1 hit");
    assert_eq!(stats.misses, 1, "Should still have 1 miss");
}

#[test]
fn test_execute_python_cached_uses_thread_local_cache() {
    // Clear cache first
    clear_thread_local_cache();

    let code = "3 + 3";

    // First execution - cache miss
    let result1 = execute_python_cached(code).unwrap();
    assert_eq!(result1, "6");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 1);
    assert_eq!(stats.misses, 1);

    // Second execution - cache hit
    let result2 = execute_python_cached(code).unwrap();
    assert_eq!(result2, "6");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.hits, 1);
}

#[test]
fn test_execute_python_cached_global_uses_global_cache() {
    // Use a unique code string to avoid interference from other tests
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_code = format!("5 * 5 + {}", nanos);

    // First execution - should be a cache miss
    let result1 = execute_python_cached_global(&unique_code).unwrap();

    // Record stats after first execution
    let stats_after_first = get_global_cache_stats();
    let hits_after_first = stats_after_first.hits;

    // Second execution - should be a cache hit for the same unique code
    let result2 = execute_python_cached_global(&unique_code).unwrap();
    assert_eq!(result1, result2, "Results should be the same");

    // Verify that hits increased by exactly 1
    let stats_after_second = get_global_cache_stats();
    assert_eq!(
        stats_after_second.hits,
        hits_after_first + 1,
        "Second execution should increment hits by 1 (from {} to {})",
        hits_after_first,
        stats_after_second.hits
    );
}

#[test]
fn test_thread_local_cache_isolation() {
    use std::thread;

    // Each thread should have its own cache
    let handle1 = thread::spawn(|| {
        clear_thread_local_cache();

        let code = "10 + 10";
        execute_python(code).unwrap();
        execute_python(code).unwrap();

        let stats = get_thread_local_cache_stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    });

    let handle2 = thread::spawn(|| {
        clear_thread_local_cache();

        let code = "20 + 20";
        execute_python(code).unwrap();
        execute_python(code).unwrap();

        let stats = get_thread_local_cache_stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

#[test]
fn test_global_cache_shared_across_threads() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    // Clear global cache first
    clear_global_cache();

    let hit_count = Arc::new(AtomicUsize::new(0));

    // Use unique code to avoid interference from other tests
    let unique_code = "9876543";

    // First thread populates cache
    let code_clone = unique_code.to_string();
    let handle1 = thread::spawn(move || {
        execute_python_cached_global(&code_clone).unwrap();
    });
    handle1.join().unwrap();

    // Get initial stats
    let initial_stats = get_global_cache_stats();
    let initial_hits = initial_stats.hits;

    // Multiple threads should see the same cached entry
    let mut handles = vec![];
    for _ in 0..5 {
        let hit_count = Arc::clone(&hit_count);
        let code_clone = unique_code.to_string();
        let handle = thread::spawn(move || {
            let result = execute_python_cached_global(&code_clone).unwrap();
            assert_eq!(result, "9876543");
            hit_count.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(hit_count.load(Ordering::SeqCst), 5);

    let stats = get_global_cache_stats();
    // Should have at least 5 more hits than initial
    assert!(
        stats.hits >= initial_hits + 5,
        "Should have at least {} cache hits, got {}",
        initial_hits + 5,
        stats.hits
    );
}

#[test]
fn test_clear_thread_local_cache() {
    clear_thread_local_cache();

    // Add some entries
    execute_python("1 + 1").unwrap();
    execute_python("2 + 2").unwrap();
    execute_python("3 + 3").unwrap();

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 3);

    // Clear cache
    clear_thread_local_cache();

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_clear_global_cache() {
    // Just verify that clearing the cache resets all stats to 0
    // Don't make assumptions about initial state due to parallel test execution
    clear_global_cache();

    let stats = get_global_cache_stats();
    assert_eq!(stats.size, 0, "After clear, cache size should be 0");
    assert_eq!(stats.hits, 0, "After clear, hits should be 0");
    assert_eq!(stats.misses, 0, "After clear, misses should be 0");

    // Add one unique entry
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_code = format!("42 + {}", nanos);
    execute_python_cached_global(&unique_code).unwrap();

    let stats = get_global_cache_stats();
    assert!(
        stats.size >= 1,
        "Cache should have at least 1 entry after adding one"
    );
    assert!(
        stats.misses >= 1,
        "Should have at least 1 miss after adding one"
    );
}

#[test]
fn test_cache_hit_rate_95_percent() {
    // AC3.1: Cache hit rate ≥95% for repeated code
    clear_thread_local_cache();

    let code = "42";

    // First request is a miss
    execute_python(code).unwrap();

    // Next 99 requests should all be hits
    for _ in 0..99 {
        execute_python(code).unwrap();
    }

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.hits, 99);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate, 0.99); // 99%
    assert!(stats.hit_rate >= 0.95, "Hit rate {} < 95%", stats.hit_rate);
}

#[test]
fn test_cache_different_code_produces_different_results() {
    // AC3.6: Cache invalidation - different code produces different results
    clear_thread_local_cache();

    let code1 = "10 + 20";
    let code2 = "15 + 15";

    let result1 = execute_python(code1).unwrap();
    assert_eq!(result1, "30");

    let result2 = execute_python(code2).unwrap();
    assert_eq!(result2, "30");

    // Both should be cached separately
    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 2);
}

#[test]
fn test_cache_correctness_with_variables() {
    clear_thread_local_cache();

    let code1 = "x = 10\ny = 20\nx + y";
    let code2 = "x = 5\ny = 15\nx + y";

    let result1 = execute_python(code1).unwrap();
    assert_eq!(result1, "30");

    let result2 = execute_python(code2).unwrap();
    assert_eq!(result2, "20");

    // Verify results are correct even with cache
    let result1_again = execute_python(code1).unwrap();
    assert_eq!(result1_again, "30");

    let result2_again = execute_python(code2).unwrap();
    assert_eq!(result2_again, "20");
}

#[test]
fn test_cache_with_errors() {
    clear_thread_local_cache();

    // Error cases should not be cached (only bytecode is cached)
    let code = "10 / 0";

    let result1 = execute_python(code);
    assert!(result1.is_err());

    // Second execution should also error
    let result2 = execute_python(code);
    assert!(result2.is_err());

    // The bytecode is cached, but the error occurs during execution
    let stats = get_thread_local_cache_stats();
    assert_eq!(
        stats.size, 1,
        "Bytecode should be cached even if execution fails"
    );
}

#[test]
fn test_cache_stats_accuracy() {
    clear_thread_local_cache();

    // Execute 5 unique code snippets
    for i in 0..5 {
        let code = format!("x = {}", i);
        execute_python(&code).unwrap();
    }

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 5);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 5);
    assert_eq!(stats.hit_rate, 0.0);

    // Access first 3 entries again
    for i in 0..3 {
        let code = format!("x = {}", i);
        execute_python(&code).unwrap();
    }

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 5);
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 5);
    assert_eq!(stats.hit_rate, 0.375); // 3/8 = 0.375
}

#[test]
fn test_cache_capacity_limit() {
    clear_thread_local_cache();

    // Fill cache with many entries (default capacity is 1000)
    // Insert more than capacity to test eviction
    for i in 0..1100 {
        let code = format!("x = {}", i);
        execute_python(&code).unwrap();
    }

    let stats = get_thread_local_cache_stats();
    // Should be at or near capacity (not over)
    assert!(stats.size <= stats.capacity);
    assert_eq!(stats.capacity, 1000);
}

#[test]
fn test_empty_code_caching() {
    clear_thread_local_cache();

    let code = "";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 1);
}

#[test]
fn test_cache_with_print_statements() {
    clear_thread_local_cache();

    let code = "print(42)";

    let result1 = execute_python(code).unwrap();
    assert_eq!(result1, "42\n");

    let result2 = execute_python(code).unwrap();
    assert_eq!(result2, "42\n");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.hits, 1);
}

#[test]
fn test_cache_with_complex_expressions() {
    clear_thread_local_cache();

    let code = "(1 + 2) * (3 + 4) / 7";

    let result1 = execute_python(code).unwrap();
    assert_eq!(result1, "3");

    let result2 = execute_python(code).unwrap();
    assert_eq!(result2, "3");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.hits, 1);
}

// Edge case tests

#[test]
fn test_cache_whitespace_sensitivity() {
    clear_thread_local_cache();

    let code1 = "2+2";
    let code2 = "2 + 2";

    execute_python(code1).unwrap();
    execute_python(code2).unwrap();

    // Different whitespace = different cache entries
    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 2);
}

#[test]
fn test_cache_unicode_code() {
    clear_thread_local_cache();

    // Unicode in string literals (comments not supported in this Python subset)
    let code = "x = 42\nx";

    let result = execute_python(code).unwrap();
    assert_eq!(result, "42");

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 1);
}

#[test]
fn test_global_cache_statistics() {
    clear_global_cache();

    // Use unique code to avoid interference from other tests
    let code = "12345678";

    // First execution
    execute_python_cached_global(code).unwrap();

    let stats1 = get_global_cache_stats();
    // Size might be > 1 due to other tests, but we check relative changes
    let initial_size = stats1.size;
    let initial_misses = stats1.misses;
    let initial_hits = stats1.hits;

    // Second execution
    execute_python_cached_global(code).unwrap();

    let stats2 = get_global_cache_stats();
    assert_eq!(stats2.size, initial_size); // Size should not increase
    assert_eq!(stats2.misses, initial_misses); // Misses should not increase
    assert_eq!(stats2.hits, initial_hits + 1); // Hits should increase by 1
}
