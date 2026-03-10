//! Integration QA: Cache ↔ Profiling ↔ Pipeline Tests
//!
//! Tests interactions between compilation cache (issue 10), profiling infrastructure (issue 11),
//! and the main execution pipeline (lib.rs)
//!
//! PRIORITY 1: Test cache integration with execute_python
//! PRIORITY 2: Test profiling integration with execute_python
//! PRIORITY 3: Test cache + profiling together
//! PRIORITY 4: Test thread-local vs global cache behavior

use pyrust::{
    clear_global_cache, clear_thread_local_cache, execute_python, execute_python_cached,
    execute_python_cached_global, get_global_cache_stats, get_thread_local_cache_stats,
    profiling::execute_python_profiled,
};

/// Test that repeated execution uses cache and produces same results
#[test]
fn test_cache_integration_repeated_execution() {
    clear_thread_local_cache();

    let code = "2 + 3 * 4";

    // First execution - cache miss
    let result1 = execute_python(code).expect("First execution failed");
    assert_eq!(result1, "14");

    // Second execution - cache hit (should be faster but same result)
    let result2 = execute_python(code).expect("Second execution failed");
    assert_eq!(result2, "14");

    // Verify cache was used
    let stats = get_thread_local_cache_stats();
    assert!(stats.hits > 0, "Cache should have at least one hit");
}

/// Test that different code produces different results (no false cache hits)
#[test]
fn test_cache_integration_collision_avoidance() {
    clear_thread_local_cache();

    let code1 = "10 + 20";
    let code2 = "15 * 2";

    let result1 = execute_python(code1).expect("Code1 failed");
    assert_eq!(result1, "30");

    let result2 = execute_python(code2).expect("Code2 failed");
    assert_eq!(result2, "30");

    // Run again - should get same results from cache
    let result1_cached = execute_python(code1).expect("Code1 cached failed");
    assert_eq!(result1_cached, "30");

    let result2_cached = execute_python(code2).expect("Code2 cached failed");
    assert_eq!(result2_cached, "30");

    // Both should be correct despite producing same value
    let stats = get_thread_local_cache_stats();
    assert!(stats.hits >= 2, "Should have cache hits for both codes");
}

/// Test cache stats are tracked correctly
#[test]
fn test_cache_integration_stats_tracking() {
    clear_thread_local_cache();

    let code = "42";

    // Execute once - should be a miss
    let _ = execute_python(code).expect("Execution failed");
    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.misses, 1, "First execution should be a miss");
    assert_eq!(stats.hits, 0, "No hits yet");

    // Execute again - should be a hit
    let _ = execute_python(code).expect("Second execution failed");
    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.hits, 1, "Second execution should be a hit");
    assert_eq!(stats.misses, 1, "Still only one miss");
}

/// Test profiling integration with basic execution
#[test]
fn test_profiling_integration_basic() {
    let code = "2 + 2";
    let (output, profile) = execute_python_profiled(code).expect("Profiled execution failed");

    assert_eq!(output, "4", "Output should be correct");

    // Verify all stages have non-zero timings
    assert!(profile.lex_ns > 0, "Lex stage should have timing");
    assert!(profile.parse_ns > 0, "Parse stage should have timing");
    assert!(profile.compile_ns > 0, "Compile stage should have timing");
    assert!(
        profile.vm_execute_ns > 0,
        "VM execute stage should have timing"
    );
    assert!(profile.format_ns > 0, "Format stage should have timing");
    assert!(profile.total_ns > 0, "Total should have timing");
}

/// Test profiling timing validation (sum ≈ total)
#[test]
fn test_profiling_integration_timing_validation() {
    let code = "x = 10\ny = 20\nx + y";
    let (output, profile) = execute_python_profiled(code).expect("Profiled execution failed");

    assert_eq!(output, "30", "Output should be correct");

    // Verify timing sum is within 5% of total
    assert!(
        profile.validate_timing_sum(),
        "Sum of stage timings should be within 5% of total"
    );
}

/// Test profiling with complex program
#[test]
fn test_profiling_integration_complex_program() {
    let code_simple = r#"
def add(a, b):
    return a + b

x = add(10, 20)
y = add(x, 30)
y
"#;
    let (output, profile) =
        execute_python_profiled(code_simple).expect("Complex profiled execution failed");

    assert_eq!(output, "60", "Output should be correct");

    // All stages should have recorded time
    let sum = profile.lex_ns
        + profile.parse_ns
        + profile.compile_ns
        + profile.vm_execute_ns
        + profile.format_ns;
    assert!(sum > 0, "Total stage time should be positive");
    assert!(
        sum <= profile.total_ns + (profile.total_ns / 20), // Allow 5% overhead
        "Sum should not significantly exceed total"
    );
}

/// Test profiling format_table output
#[test]
fn test_profiling_integration_table_format() {
    let code = "42";
    let (_, profile) = execute_python_profiled(code).expect("Profiled execution failed");

    let table = profile.format_table();

    // Verify table contains expected headers
    assert!(table.contains("Stage"), "Table should have Stage column");
    assert!(table.contains("Time(ns)"), "Table should have Time column");
    assert!(
        table.contains("Percent"),
        "Table should have Percent column"
    );

    // Verify all stage names appear
    assert!(table.contains("Lex"), "Table should show Lex stage");
    assert!(table.contains("Parse"), "Table should show Parse stage");
    assert!(table.contains("Compile"), "Table should show Compile stage");
    assert!(
        table.contains("VM Execute"),
        "Table should show VM Execute stage"
    );
    assert!(table.contains("Format"), "Table should show Format stage");
    assert!(table.contains("TOTAL"), "Table should show TOTAL");
}

/// Test profiling JSON format
#[test]
fn test_profiling_integration_json_format() {
    let code = "10 + 20";
    let (_, profile) = execute_python_profiled(code).expect("Profiled execution failed");

    let json = profile.format_json();

    // Verify JSON contains all required fields
    assert!(json.contains("lex_ns"), "JSON should have lex_ns field");
    assert!(json.contains("parse_ns"), "JSON should have parse_ns field");
    assert!(
        json.contains("compile_ns"),
        "JSON should have compile_ns field"
    );
    assert!(
        json.contains("vm_execute_ns"),
        "JSON should have vm_execute_ns field"
    );
    assert!(
        json.contains("format_ns"),
        "JSON should have format_ns field"
    );
    assert!(json.contains("total_ns"), "JSON should have total_ns field");

    // Verify it's valid JSON-like structure (escape braces in format string)
    assert!(
        json.starts_with("{"),
        "JSON should start with opening brace"
    );
    assert!(json.ends_with("}"), "JSON should end with closing brace");
}

/// Test cache clear functionality
#[test]
fn test_cache_integration_clear() {
    clear_thread_local_cache();

    let code = "100";

    // Execute and cache
    let _ = execute_python(code).expect("Execution failed");
    let stats1 = get_thread_local_cache_stats();
    assert!(stats1.size > 0, "Cache should have entries");

    // Clear cache
    clear_thread_local_cache();
    let stats2 = get_thread_local_cache_stats();
    assert_eq!(stats2.size, 0, "Cache should be empty after clear");
    assert_eq!(stats2.hits, 0, "Hits should be reset");
    assert_eq!(stats2.misses, 0, "Misses should be reset");
}

/// Test thread-local vs global cache isolation
#[test]
fn test_cache_integration_thread_local_vs_global() {
    clear_thread_local_cache();
    clear_global_cache();

    let code = "7 * 8";

    // Use thread-local cache
    let _ = execute_python_cached(code).expect("Thread-local execution failed");
    let tl_stats = get_thread_local_cache_stats();
    let global_stats = get_global_cache_stats();

    assert_eq!(tl_stats.size, 1, "Thread-local cache should have 1 entry");
    assert_eq!(global_stats.size, 0, "Global cache should be empty");

    // Use global cache
    let _ = execute_python_cached_global(code).expect("Global execution failed");
    let global_stats2 = get_global_cache_stats();
    assert_eq!(
        global_stats2.size, 1,
        "Global cache should now have 1 entry"
    );
}

/// Test cache with profiling (ensure they don't interfere)
#[test]
fn test_cache_profiling_integration_combined() {
    clear_thread_local_cache();

    let code = "5 + 10";

    // First: regular cached execution
    let result1 = execute_python(code).expect("Cached execution failed");
    assert_eq!(result1, "15");

    // Second: profiled execution (bypasses cache)
    let (result2, profile) = execute_python_profiled(code).expect("Profiled execution failed");
    assert_eq!(result2, "15");
    assert!(profile.total_ns > 0, "Profile should have timing");

    // Third: cached execution again (should still work)
    let result3 = execute_python(code).expect("Second cached execution failed");
    assert_eq!(result3, "15");
}

/// Test cache hit rate calculation
#[test]
fn test_cache_integration_hit_rate() {
    clear_thread_local_cache();

    let code = "123";

    // Execute 5 times
    for _ in 0..5 {
        let _ = execute_python(code).expect("Execution failed");
    }

    let stats = get_thread_local_cache_stats();
    assert_eq!(
        stats.misses, 1,
        "Should have exactly 1 miss (first execution)"
    );
    assert_eq!(stats.hits, 4, "Should have 4 hits (subsequent executions)");

    // Hit rate should be 80% (4/5)
    let expected_rate = 0.8;
    assert!(
        (stats.hit_rate - expected_rate).abs() < 0.01,
        "Hit rate should be approximately 80%, got {}",
        stats.hit_rate
    );
}

/// Test profiling overhead is minimal
#[test]
fn test_profiling_integration_low_overhead() {
    let code = "42";

    // Run profiled version multiple times
    for _ in 0..10 {
        let (output, profile) = execute_python_profiled(code).expect("Profiled execution failed");
        assert_eq!(output, "42");

        // Verify timing sum validation passes (overhead < 5%)
        assert!(
            profile.validate_timing_sum(),
            "Profiling overhead should be < 5%"
        );
    }
}

/// Test cache memory usage with multiple entries
#[test]
fn test_cache_integration_multiple_entries() {
    clear_thread_local_cache();

    // Insert multiple different programs
    for i in 0..10 {
        let code = format!("{} + {}", i, i + 1);
        let expected = format!("{}", i + i + 1);
        let result = execute_python(&code).expect("Execution failed");
        assert_eq!(result, expected);
    }

    let stats = get_thread_local_cache_stats();
    assert_eq!(stats.size, 10, "Should have 10 cached entries");
    assert_eq!(stats.misses, 10, "Should have 10 misses (all unique)");
    assert_eq!(stats.hits, 0, "Should have 0 hits (no repeats)");
}
