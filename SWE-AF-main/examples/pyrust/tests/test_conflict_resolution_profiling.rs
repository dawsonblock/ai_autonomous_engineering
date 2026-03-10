//! Integration tests for profiling.rs conflict resolution
//!
//! Tests the interaction between:
//! - issue/05-profiling-cast-warning-fix: .unsigned_abs() usage
//! - issue/06-code-formatting: Multi-line formatting
//!
//! Priority: HIGH - This is a conflict resolution area
//!
//! Key risks:
//! - unsigned_abs() arithmetic correctness
//! - Timing validation with 5% threshold
//! - Multi-line sum calculation accuracy

use pyrust::profiling::{execute_python_profiled, PipelineProfile};

#[test]
fn test_profiling_timing_validation_unsigned_abs() {
    // Test that unsigned_abs() correctly calculates timing differences
    // This is the core conflict resolution: .abs() as u64 -> .unsigned_abs()
    let (output, profile) = execute_python_profiled("2+3").unwrap();

    assert_eq!(output, "5");

    // Verify that validate_timing_sum uses unsigned_abs correctly
    let sum = profile.lex_ns
        + profile.parse_ns
        + profile.compile_ns
        + profile.vm_execute_ns
        + profile.format_ns;

    // Calculate diff using unsigned_abs (the conflict resolution change)
    let diff = (sum as i64 - profile.total_ns as i64).unsigned_abs();
    let threshold = (profile.total_ns as f64 * 0.05) as u64;

    // Verify calculation matches
    assert!(
        diff <= threshold,
        "unsigned_abs calculation failed: diff={}, threshold={}",
        diff,
        threshold
    );

    // Also verify through the method
    assert!(
        profile.validate_timing_sum(),
        "validate_timing_sum should pass with unsigned_abs"
    );
}

#[test]
fn test_profiling_unsigned_abs_with_negative_diff() {
    // Test unsigned_abs when sum < total (negative difference)
    // Set up scenario where sum < total
    let profile = PipelineProfile {
        lex_ns: 1000,
        parse_ns: 2000,
        compile_ns: 3000,
        vm_execute_ns: 4000,
        format_ns: 5000,
        total_ns: 16000, // Sum is 15000, so diff = -1000
    };

    // With unsigned_abs, abs(-1000) = 1000
    // Threshold = 16000 * 0.05 = 800
    // 1000 > 800, so should fail
    assert!(
        !profile.validate_timing_sum(),
        "Should fail when diff exceeds threshold"
    );
}

#[test]
fn test_profiling_unsigned_abs_with_positive_diff() {
    // Test unsigned_abs when sum > total (positive difference)
    // Set up scenario where sum > total
    let profile = PipelineProfile {
        lex_ns: 1000,
        parse_ns: 2000,
        compile_ns: 3000,
        vm_execute_ns: 4000,
        format_ns: 5000,
        total_ns: 14000, // Sum is 15000, so diff = +1000
    };

    // With unsigned_abs, abs(1000) = 1000
    // Threshold = 14000 * 0.05 = 700
    // 1000 > 700, so should fail
    assert!(
        !profile.validate_timing_sum(),
        "Should fail when sum exceeds total by >5%"
    );
}

#[test]
fn test_profiling_unsigned_abs_boundary_cases() {
    // Test exact boundary at 5% threshold
    let mut profile = PipelineProfile {
        lex_ns: 1000,
        parse_ns: 2000,
        compile_ns: 3000,
        vm_execute_ns: 4000,
        format_ns: 5000,
        total_ns: 15000, // Exact match, diff = 0
    };

    assert!(
        profile.validate_timing_sum(),
        "Should pass with exact match"
    );

    // Test at exactly 5% threshold
    profile.total_ns = 15790; // Sum=15000, diff=790, threshold=789.5, should fail
    assert!(
        !profile.validate_timing_sum(),
        "Should fail at >5% threshold"
    );

    profile.total_ns = 15750; // Sum=15000, diff=750, threshold=787.5, should pass
    assert!(
        profile.validate_timing_sum(),
        "Should pass at <5% threshold"
    );
}

#[test]
fn test_profiling_all_stages_execute() {
    // Test that all pipeline stages execute and have timing data
    let test_cases = vec![
        ("2+3", "5"),
        ("print(42)", "42\n"),
        ("x=10\ny=20\nx+y", "30"),
        ("def f():\n    return 123\nf()", "123"),
    ];

    for (code, expected_output) in test_cases {
        let (output, profile) = execute_python_profiled(code).unwrap();
        assert_eq!(output, expected_output);

        // All stages should have executed (non-zero time)
        assert!(profile.lex_ns > 0, "Lex stage should execute");
        assert!(profile.parse_ns > 0, "Parse stage should execute");
        assert!(profile.compile_ns > 0, "Compile stage should execute");
        assert!(profile.vm_execute_ns > 0, "VM execute stage should execute");
        assert!(profile.format_ns > 0, "Format stage should execute");
        assert!(profile.total_ns > 0, "Total time should be non-zero");

        // Verify unsigned_abs validation
        assert!(
            profile.validate_timing_sum(),
            "Timing sum validation failed for: {}",
            code
        );
    }
}

#[test]
fn test_profiling_format_table_contains_all_stages() {
    // Test formatting output (multi-line formatting from conflict resolution)
    let (_, profile) = execute_python_profiled("42").unwrap();
    let table = profile.format_table();

    // Verify all stages are present
    assert!(table.contains("Lex"));
    assert!(table.contains("Parse"));
    assert!(table.contains("Compile"));
    assert!(table.contains("VM Execute"));
    assert!(table.contains("Format"));
    assert!(table.contains("TOTAL"));

    // Verify table structure
    assert!(table.contains("Stage Breakdown:"));
    assert!(table.contains("│ Stage"));
    assert!(table.contains("│ Time(ns)"));
    assert!(table.contains("│ Percent"));
}

#[test]
fn test_profiling_format_json_structure() {
    // Test JSON formatting
    let (_, profile) = execute_python_profiled("1+1").unwrap();
    let json = profile.format_json();

    // Verify all fields are present
    assert!(json.contains("\"lex_ns\":"));
    assert!(json.contains("\"parse_ns\":"));
    assert!(json.contains("\"compile_ns\":"));
    assert!(json.contains("\"vm_execute_ns\":"));
    assert!(json.contains("\"format_ns\":"));
    assert!(json.contains("\"total_ns\":"));

    // Verify JSON structure
    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
}

#[test]
fn test_profiling_empty_code() {
    // Test profiling with empty code
    let (output, profile) = execute_python_profiled("").unwrap();
    assert_eq!(output, "");

    // Even empty code should have timing
    assert!(profile.total_ns > 0);
}

#[test]
fn test_profiling_error_propagation() {
    // Test that errors are properly propagated through profiling
    let result = execute_python_profiled("1 / 0");
    assert!(result.is_err(), "Division by zero should error");

    let result = execute_python_profiled("x = @");
    assert!(result.is_err(), "Syntax error should propagate");

    let result = execute_python_profiled("undefined_var");
    assert!(result.is_err(), "Undefined variable should error");
}

#[test]
fn test_profiling_complex_code_timing_accuracy() {
    // Test timing accuracy with complex code
    let complex_code = r#"
def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

result = factorial(5)
print(result)
"#;

    let (output, profile) = execute_python_profiled(complex_code).unwrap();
    assert_eq!(output, "120\n");

    // Verify timing validation with unsigned_abs
    assert!(
        profile.validate_timing_sum(),
        "Complex code timing validation failed"
    );

    // VM execute should take longer for recursive code
    assert!(profile.vm_execute_ns > 0);

    // Total should be sum of all stages (within 5%)
    let sum = profile.lex_ns
        + profile.parse_ns
        + profile.compile_ns
        + profile.vm_execute_ns
        + profile.format_ns;
    let diff_pct = ((sum as i64 - profile.total_ns as i64).unsigned_abs() as f64
        / profile.total_ns as f64)
        * 100.0;
    assert!(
        diff_pct <= 5.0,
        "Timing difference {}% exceeds 5% threshold",
        diff_pct
    );
}

#[test]
fn test_profiling_multiple_executions_consistency() {
    // Test that unsigned_abs calculation is consistent across multiple runs
    let code = "2 ** 10"; // Power operation

    for _ in 0..5 {
        let (output, profile) = execute_python_profiled(code).unwrap();
        assert_eq!(output, "1024");
        assert!(
            profile.validate_timing_sum(),
            "Timing validation should be consistent"
        );
    }
}

#[test]
fn test_profiling_zero_total_edge_case() {
    // Test edge case where total is 0 (shouldn't happen but defensive)
    let profile = PipelineProfile::default();
    // All zeros
    assert!(
        profile.validate_timing_sum(),
        "All zeros should validate (0 <= 0)"
    );
}

#[test]
fn test_profiling_percentage_calculation_in_table() {
    // Test that percentage calculation works correctly
    let (_, profile) = execute_python_profiled("2+2").unwrap();
    let table = profile.format_table();

    // Total should show 100.00%
    assert!(table.contains("100.0"));

    // Each stage should have a percentage
    // Parse the table and verify percentages sum to ~100%
    let lines: Vec<&str> = table.lines().collect();
    let mut total_pct = 0.0;

    for line in lines {
        if line.contains("│") && !line.contains("Stage") && !line.contains("─") {
            // Extract percentage (format: "XX.XX%")
            if let Some(pct_start) = line.rfind('%') {
                if let Some(num_start) = line[..pct_start].rfind(' ') {
                    let pct_str = line[num_start + 1..pct_start].trim();
                    if let Ok(pct) = pct_str.parse::<f64>() {
                        if !line.contains("TOTAL") {
                            total_pct += pct;
                        }
                    }
                }
            }
        }
    }

    // Stage percentages should sum to ~100%
    assert!(
        (total_pct - 100.0).abs() < 0.1,
        "Stage percentages should sum to ~100%, got {}",
        total_pct
    );
}
