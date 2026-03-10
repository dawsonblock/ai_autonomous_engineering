//! Integration tests for conflict resolution areas
//!
//! This test suite focuses on verifying that merged code from different branches
//! works correctly, especially at conflict resolution points.

use pyrust::{
    cache::CompilationCache,
    daemon_protocol::{DaemonRequest, DaemonResponse},
    execute_python,
    profiling::execute_python_profiled,
};
use std::sync::Arc;

/// Test Priority 1: Cargo.toml conflict - Binary optimization + Cache + Profiling benchmarks
/// Verifies that all three benchmark configurations can coexist
#[test]
fn test_cargo_toml_benchmarks_coexist() {
    // This test verifies the Cargo.toml structure is valid by checking that
    // the project compiles successfully (which it does if this test runs)

    // We can indirectly verify by ensuring all three features work:

    // 1. Binary optimization (release profile settings)
    let result = execute_python("2 + 3").unwrap();
    assert_eq!(result, "5");

    // 2. Cache performance (cache module)
    let mut cache = CompilationCache::new(10);
    cache.insert(
        "test".to_string(),
        Arc::new(
            pyrust::compiler::compile(
                &pyrust::parser::parse(pyrust::lexer::lex("42").unwrap()).unwrap(),
            )
            .unwrap(),
        ),
    );
    assert!(cache.get("test").is_some());

    // 3. Profiling overhead (profiling module)
    let (output, _profile) = execute_python_profiled("5 + 5").unwrap();
    assert_eq!(output, "10");
}

/// Test Priority 1: src/lib.rs conflict - daemon_protocol + profiling modules
/// Verifies both modules are accessible and work independently
#[test]
fn test_lib_rs_modules_coexist() {
    // Test daemon_protocol module
    let request = DaemonRequest::new("2+3");
    let encoded = request.encode();
    let (decoded, _) = DaemonRequest::decode(&encoded).unwrap();
    assert_eq!(decoded.code(), "2+3");

    let response = DaemonResponse::success("5");
    let encoded = response.encode();
    let (decoded, _) = DaemonResponse::decode(&encoded).unwrap();
    assert_eq!(decoded.output(), "5");
    assert!(decoded.is_success());

    // Test profiling module
    let (output, profile) = execute_python_profiled("10 + 20").unwrap();
    assert_eq!(output, "30");
    assert!(profile.total_ns > 0);
    assert!(profile.validate_timing_sum());
}

/// Test Priority 2: Cache + Profiling interaction
/// Verifies that profiling works correctly even when cache is used
#[test]
fn test_cache_profiling_interaction() {
    // Execute same code twice - second time should use cache
    let code = "x = 100\ny = 200\nx + y";

    // First execution - cache miss
    let (output1, profile1) = execute_python_profiled(code).unwrap();
    assert_eq!(output1, "300");
    assert!(profile1.lex_ns > 0);
    assert!(profile1.parse_ns > 0);
    assert!(profile1.compile_ns > 0);

    // Second execution - cache hit (profiling should still work)
    let (output2, profile2) = execute_python_profiled(code).unwrap();
    assert_eq!(output2, "300");
    assert!(profile2.total_ns > 0);

    // Both executions should produce correct output
    assert_eq!(output1, output2);
}

/// Test Priority 2: Cache + Daemon Protocol interaction
/// Verifies cache works correctly with daemon request/response encoding
#[test]
fn test_cache_daemon_protocol_interaction() {
    let code = "42";

    // Execute through normal path (uses cache)
    let result1 = execute_python(code).unwrap();
    assert_eq!(result1, "42");

    // Encode as daemon request
    let request = DaemonRequest::new(code);
    let encoded_req = request.encode();
    let (decoded_req, _) = DaemonRequest::decode(&encoded_req).unwrap();

    // Execute decoded request code (should use cache)
    let result2 = execute_python(decoded_req.code()).unwrap();
    assert_eq!(result2, "42");

    // Encode as daemon response
    let response = DaemonResponse::success(&result2);
    let encoded_resp = response.encode();
    let (decoded_resp, _) = DaemonResponse::decode(&encoded_resp).unwrap();

    assert_eq!(decoded_resp.output(), "42");
    assert!(decoded_resp.is_success());
}

/// Test Priority 2: Binary optimization + Cache interaction
/// Verifies release build settings don't break cache functionality
#[test]
fn test_binary_optimization_cache_interaction() {
    let cache = CompilationCache::new(100);

    // Test cache with various code samples
    let test_cases = vec![
        ("2 + 2", "4"),
        ("10 * 5", "50"),
        ("x = 100\nx", "100"),
        ("print(42)", "42\n"),
    ];

    for (code, expected) in test_cases {
        // First execution - cache miss
        let result1 = execute_python(code).unwrap();
        assert_eq!(result1, expected);

        // Verify cache statistics
        let stats = cache.stats();
        assert!(stats.capacity > 0);

        // Second execution - cache hit
        let result2 = execute_python(code).unwrap();
        assert_eq!(result2, expected);
        assert_eq!(result1, result2);
    }
}

/// Test Priority 2: Binary optimization + Profiling interaction
/// Verifies LTO and optimization settings don't affect profiling accuracy
#[test]
fn test_binary_optimization_profiling_interaction() {
    let code = "a = 10\nb = 20\nc = a + b\nprint(c)\nc";

    let (output, profile) = execute_python_profiled(code).unwrap();
    assert_eq!(output, "30\n30");

    // Verify all stages are measured
    assert!(profile.lex_ns > 0, "Lex stage should have non-zero time");
    assert!(
        profile.parse_ns > 0,
        "Parse stage should have non-zero time"
    );
    assert!(
        profile.compile_ns > 0,
        "Compile stage should have non-zero time"
    );
    assert!(
        profile.vm_execute_ns > 0,
        "VM execute stage should have non-zero time"
    );
    assert!(
        profile.format_ns > 0,
        "Format stage should have non-zero time"
    );

    // Verify timing sum is within 5% of total (AC5.2)
    assert!(
        profile.validate_timing_sum(),
        "Sum of stage timings should be within 5% of total"
    );

    // Verify profiling overhead is minimal (stages sum to reasonable total)
    let sum = profile.lex_ns
        + profile.parse_ns
        + profile.compile_ns
        + profile.vm_execute_ns
        + profile.format_ns;
    assert!(sum <= profile.total_ns, "Stage sum should not exceed total");
}

/// Test: Daemon protocol with error handling
/// Verifies error responses work correctly
#[test]
fn test_daemon_protocol_error_handling() {
    // Create error response
    let error_response = DaemonResponse::error("Division by zero");
    let encoded = error_response.encode();
    let (decoded, _) = DaemonResponse::decode(&encoded).unwrap();

    assert!(decoded.is_error());
    assert!(!decoded.is_success());
    assert_eq!(decoded.output(), "Division by zero");
}

/// Test: Cache collision detection with daemon protocol
/// Verifies different code produces different results even when cached
#[test]
fn test_cache_collision_with_daemon_protocol() {
    let code1 = "10 + 20";
    let code2 = "15 + 15";

    // Execute both codes
    let result1 = execute_python(code1).unwrap();
    let result2 = execute_python(code2).unwrap();

    // Both produce same result value
    assert_eq!(result1, "30");
    assert_eq!(result2, "30");

    // Encode as daemon requests
    let req1 = DaemonRequest::new(code1);
    let req2 = DaemonRequest::new(code2);

    // Verify they encode differently
    assert_ne!(req1.encode(), req2.encode());

    // Execute again - cache should not confuse them
    let result1_again = execute_python(code1).unwrap();
    let result2_again = execute_python(code2).unwrap();

    assert_eq!(result1_again, "30");
    assert_eq!(result2_again, "30");
}

/// Test: Profiling with daemon protocol
/// Verifies profiling data can be transmitted via daemon protocol
#[test]
fn test_profiling_with_daemon_protocol() {
    let code = "2 + 3";

    // Get profiling data
    let (_output, profile) = execute_python_profiled(code).unwrap();

    // Format as JSON (could be transmitted via daemon protocol)
    let json = profile.format_json();
    assert!(json.contains("\"lex_ns\":"));
    assert!(json.contains("\"parse_ns\":"));
    assert!(json.contains("\"compile_ns\":"));
    assert!(json.contains("\"vm_execute_ns\":"));
    assert!(json.contains("\"format_ns\":"));
    assert!(json.contains("\"total_ns\":"));

    // Format as table (could be transmitted via daemon protocol)
    let table = profile.format_table();
    assert!(table.contains("Stage Breakdown:"));
    assert!(table.contains("Lex"));
    assert!(table.contains("Parse"));
    assert!(table.contains("Compile"));
    assert!(table.contains("VM Execute"));
    assert!(table.contains("Format"));
}

/// Test: Complex interaction - all features together
/// Verifies cache, profiling, and daemon protocol work together
#[test]
fn test_all_features_integration() {
    let code = "x = 42\nprint(x)\nx * 2";

    // 1. Execute with profiling (first time - cache miss)
    let (output1, profile1) = execute_python_profiled(code).unwrap();
    assert_eq!(output1, "42\n84");
    assert!(profile1.total_ns > 0);

    // 2. Encode as daemon request
    let request = DaemonRequest::new(code);
    let encoded_req = request.encode();
    let (decoded_req, _) = DaemonRequest::decode(&encoded_req).unwrap();
    assert_eq!(decoded_req.code(), code);

    // 3. Execute decoded request (cache hit)
    let output2 = execute_python(decoded_req.code()).unwrap();
    assert_eq!(output2, "42\n84");

    // 4. Create success response
    let response = DaemonResponse::success(&output2);
    let encoded_resp = response.encode();
    let (decoded_resp, _) = DaemonResponse::decode(&encoded_resp).unwrap();
    assert_eq!(decoded_resp.output(), "42\n84");
    assert!(decoded_resp.is_success());

    // 5. Profile again (cache hit)
    let (output3, profile2) = execute_python_profiled(code).unwrap();
    assert_eq!(output3, "42\n84");
    assert!(profile2.total_ns > 0);

    // All outputs should be identical
    assert_eq!(output1, output2);
    assert_eq!(output2, output3);
}

/// Test: Release profile settings don't break functionality
/// Verifies LTO, codegen-units=1, strip, panic=abort don't cause issues
#[test]
fn test_release_profile_compatibility() {
    // Test various operations that could be affected by optimization settings

    // 1. Function calls (could be affected by LTO)
    let code = r#"
def add(x, y):
    return x + y
add(10, 20)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "30");

    // 2. Error handling (could be affected by panic=abort)
    let result = execute_python("1 / 0");
    assert!(result.is_err());

    // 3. String operations (could be affected by optimizations)
    let result = execute_python("x = 100\nprint(x)\nx").unwrap();
    assert_eq!(result, "100\n100");

    // 4. Cache operations (could be affected by strip)
    let result1 = execute_python("2 + 2").unwrap();
    let result2 = execute_python("2 + 2").unwrap();
    assert_eq!(result1, result2);
}

/// Test: Negative numbers across cache and profiling
/// Verifies negative number parsing works with cached/profiled execution
#[test]
fn test_negative_numbers_integration() {
    let code = "x = -42\ny = -10\nx + y";

    // Execute with profiling
    let (output, profile) = execute_python_profiled(code).unwrap();
    assert_eq!(output, "-52");
    assert!(profile.total_ns > 0);

    // Execute again (cache hit)
    let result = execute_python(code).unwrap();
    assert_eq!(result, "-52");
}

/// Test: Function parameter handling across all features
/// Verifies function parameter fixes work with cache/profiling/daemon
#[test]
fn test_function_parameters_integration() {
    let code = r#"
def multiply(a, b):
    return a * b
multiply(6, 7)
"#;

    // Test with profiling
    let (output1, profile) = execute_python_profiled(code).unwrap();
    assert_eq!(output1, "42");
    assert!(profile.total_ns > 0);

    // Test with daemon protocol
    let request = DaemonRequest::new(code);
    let (decoded, _) = DaemonRequest::decode(&request.encode()).unwrap();
    let output2 = execute_python(decoded.code()).unwrap();
    assert_eq!(output2, "42");

    // Test with cache (execute again)
    let output3 = execute_python(code).unwrap();
    assert_eq!(output3, "42");
}

/// Test: Benchmark stability doesn't interfere with cache
/// Verifies benchmark configurations work with cache
#[test]
fn test_benchmark_stability_with_cache() {
    // Execute same code multiple times (benchmark-like workload)
    let code = "x = 10\nx * x";

    let mut results = Vec::new();
    for _ in 0..10 {
        let result = execute_python(code).unwrap();
        results.push(result);
    }

    // All results should be identical (stability)
    assert!(results.iter().all(|r| r == "100"));
}
