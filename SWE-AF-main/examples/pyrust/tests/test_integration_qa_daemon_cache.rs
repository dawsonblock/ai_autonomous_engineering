//! Integration QA: Daemon ↔ Cache Tests
//!
//! Tests interactions between daemon mode (issue 19) and compilation cache (issue 20).
//! The critical integration point is that the daemon server uses execute_python_cached_global
//! which accesses the GLOBAL_CACHE mutex to share compiled bytecode across all daemon requests.
//!
//! PRIORITY 1: Test daemon uses global cache (not thread-local)
//! PRIORITY 2: Test cache hit rate improves daemon performance
//! PRIORITY 3: Test concurrent daemon requests share cache correctly

use pyrust::{clear_global_cache, execute_python_cached_global, get_global_cache_stats};

/// Test that global cache is used correctly by daemon execution path
#[test]
fn test_daemon_cache_integration_global_cache_usage() {
    clear_global_cache();

    let code = "2 + 3";

    // First execution - cache miss
    let result1 = execute_python_cached_global(code).expect("First global execution failed");
    assert_eq!(result1, "5");

    let stats = get_global_cache_stats();
    assert_eq!(stats.misses, 1, "First execution should be a miss");
    assert_eq!(stats.hits, 0, "No hits yet");
    assert_eq!(stats.size, 1, "Cache should have 1 entry");

    // Second execution - cache hit
    let result2 = execute_python_cached_global(code).expect("Second global execution failed");
    assert_eq!(result2, "5");

    let stats = get_global_cache_stats();
    assert_eq!(stats.hits, 1, "Second execution should be a hit");
    assert_eq!(stats.misses, 1, "Still only one miss");

    // Third execution - another cache hit
    let result3 = execute_python_cached_global(code).expect("Third global execution failed");
    assert_eq!(result3, "5");

    let stats = get_global_cache_stats();
    assert_eq!(stats.hits, 2, "Third execution should be a hit");
    assert_eq!(stats.misses, 1, "Still only one miss");
}

/// Test cache hit rate with repeated daemon-style requests
#[test]
fn test_daemon_cache_integration_hit_rate() {
    clear_global_cache();

    let code = "42";

    // Simulate 100 daemon requests with same code
    for _ in 0..100 {
        let result = execute_python_cached_global(code).expect("Execution failed");
        assert_eq!(result, "42");
    }

    let stats = get_global_cache_stats();
    assert_eq!(
        stats.misses, 1,
        "Should have exactly 1 miss (first request)"
    );
    assert_eq!(stats.hits, 99, "Should have 99 hits (subsequent requests)");

    // Hit rate should be 99% (99/100)
    let expected_rate = 0.99;
    assert!(
        (stats.hit_rate - expected_rate).abs() < 0.01,
        "Hit rate should be approximately 99%, got {}",
        stats.hit_rate
    );
}

/// Test cache collision detection with global cache
#[test]
fn test_daemon_cache_integration_collision_detection() {
    clear_global_cache();

    // Two different code snippets that both evaluate to 30
    let code1 = "10 + 20";
    let code2 = "15 * 2";

    // Execute first code twice
    let result1_first = execute_python_cached_global(code1).expect("Code1 first execution failed");
    assert_eq!(result1_first, "30");

    let result1_second =
        execute_python_cached_global(code1).expect("Code1 second execution failed");
    assert_eq!(result1_second, "30");

    // Execute second code twice
    let result2_first = execute_python_cached_global(code2).expect("Code2 first execution failed");
    assert_eq!(result2_first, "30");

    let result2_second =
        execute_python_cached_global(code2).expect("Code2 second execution failed");
    assert_eq!(result2_second, "30");

    // Execute first code again
    let result1_third = execute_python_cached_global(code1).expect("Code1 third execution failed");
    assert_eq!(result1_third, "30");

    // All results should be correct - cache didn't confuse the two different codes
    let stats = get_global_cache_stats();
    assert_eq!(stats.size, 2, "Should have 2 cached entries");
    assert_eq!(
        stats.misses, 2,
        "Should have 2 misses (one per unique code)"
    );
    assert!(stats.hits >= 3, "Should have at least 3 hits");
}

/// Test cache with different code patterns (simulating realistic daemon workload)
#[test]
fn test_daemon_cache_integration_mixed_workload() {
    clear_global_cache();

    let test_cases = vec![
        ("2 + 3", "5"),
        ("10 * 5", "50"),
        ("100 / 2", "50"),
        ("7 - 3", "4"),
        ("8 % 3", "2"),
    ];

    // First pass - all cache misses
    for (code, expected) in &test_cases {
        let result =
            execute_python_cached_global(code).expect(&format!("Failed to execute: {}", code));
        assert_eq!(result, *expected, "Wrong result for: {}", code);
    }

    let stats_after_first = get_global_cache_stats();
    assert_eq!(stats_after_first.size, 5, "Should have 5 cached entries");
    assert_eq!(stats_after_first.misses, 5, "Should have 5 misses");

    // Second pass - all cache hits
    for (code, expected) in &test_cases {
        let result =
            execute_python_cached_global(code).expect(&format!("Failed to execute: {}", code));
        assert_eq!(result, *expected, "Wrong result for: {}", code);
    }

    let stats_after_second = get_global_cache_stats();
    assert_eq!(
        stats_after_second.hits, 5,
        "Should have 5 hits from second pass"
    );
    assert_eq!(stats_after_second.misses, 5, "Still only 5 misses");
}

/// Test cache with complex programs
#[test]
fn test_daemon_cache_integration_complex_programs() {
    clear_global_cache();

    let complex_code = "x = 10\ny = 20\nz = x + y\nz";

    // Execute multiple times
    for _ in 0..10 {
        let result =
            execute_python_cached_global(complex_code).expect("Complex code execution failed");
        assert_eq!(result, "30");
    }

    let stats = get_global_cache_stats();
    assert_eq!(stats.misses, 1, "Should have 1 miss");
    assert_eq!(stats.hits, 9, "Should have 9 hits");
}

/// Test cache with error-producing code (errors should not prevent caching)
#[test]
fn test_daemon_cache_integration_with_errors() {
    clear_global_cache();

    // Code that causes runtime error
    let error_code = "10 / 0";

    // First execution - cache miss, runtime error
    let result1 = execute_python_cached_global(error_code);
    assert!(result1.is_err(), "Should error on division by zero");

    // Second execution - cache hit (bytecode cached), same runtime error
    let result2 = execute_python_cached_global(error_code);
    assert!(result2.is_err(), "Should still error on division by zero");

    let stats = get_global_cache_stats();
    // Bytecode should be cached even for error-producing code
    assert!(stats.size >= 1, "Bytecode should be cached");
}

/// Test cache behavior with very similar code
#[test]
fn test_daemon_cache_integration_similar_code() {
    clear_global_cache();

    let codes = vec![
        ("x = 1", ""),
        ("x = 2", ""),
        ("x = 3", ""),
        ("x = 4", ""),
        ("x = 5", ""),
    ];

    // Execute each code twice
    for (code, expected) in &codes {
        let result1 =
            execute_python_cached_global(code).expect(&format!("Failed first execution: {}", code));
        assert_eq!(result1, *expected);

        let result2 = execute_python_cached_global(code)
            .expect(&format!("Failed second execution: {}", code));
        assert_eq!(result2, *expected);
    }

    let stats = get_global_cache_stats();
    assert_eq!(stats.size, 5, "Should have 5 different cached entries");
    assert_eq!(
        stats.misses, 5,
        "Should have 5 misses (one per unique code)"
    );
    assert_eq!(stats.hits, 5, "Should have 5 hits (second pass)");
}

/// Test cache clear functionality for global cache
#[test]
fn test_daemon_cache_integration_clear() {
    clear_global_cache();

    let code = "100 + 200";

    // Execute and cache
    let _ = execute_python_cached_global(code).expect("Execution failed");
    let stats1 = get_global_cache_stats();
    assert!(stats1.size > 0, "Cache should have entries");

    // Clear cache
    clear_global_cache();
    let stats2 = get_global_cache_stats();
    assert_eq!(stats2.size, 0, "Cache should be empty after clear");
    assert_eq!(stats2.hits, 0, "Hits should be reset");
    assert_eq!(stats2.misses, 0, "Misses should be reset");
}

/// Test cache with print statements
#[test]
fn test_daemon_cache_integration_print_statements() {
    clear_global_cache();

    let code = "print(42)";

    // Execute multiple times
    for _ in 0..5 {
        let result = execute_python_cached_global(code).expect("Print execution failed");
        assert_eq!(result, "42\n");
    }

    let stats = get_global_cache_stats();
    assert_eq!(stats.misses, 1, "Should have 1 miss");
    assert_eq!(stats.hits, 4, "Should have 4 hits");
}

/// Test cache performance improvement (simulating benchmark scenario)
#[test]
fn test_daemon_cache_integration_performance_pattern() {
    clear_global_cache();

    // Simulate realistic daemon workload with code reuse
    let common_codes = vec!["1", "2", "3", "4", "5"];

    // 50 requests with high code reuse (10 iterations of 5 codes)
    for _ in 0..10 {
        for code in &common_codes {
            let result =
                execute_python_cached_global(code).expect(&format!("Failed to execute: {}", code));
            assert_eq!(result, *code);
        }
    }

    let stats = get_global_cache_stats();
    assert_eq!(stats.size, 5, "Should have 5 cached entries");
    assert_eq!(stats.misses, 5, "Should have 5 misses (first iteration)");
    assert_eq!(
        stats.hits, 45,
        "Should have 45 hits (9 iterations * 5 codes)"
    );

    // Hit rate should be 90% (45/50)
    let expected_rate = 0.9;
    assert!(
        (stats.hit_rate - expected_rate).abs() < 0.01,
        "Hit rate should be approximately 90%, got {}",
        stats.hit_rate
    );
}
