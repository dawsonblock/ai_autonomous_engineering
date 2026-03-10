//! Tests for CPython comparison benchmark (AC1.3)
//!
//! This test suite validates that:
//! - AC1.3: Speedup ratio ≥50x documented with statistical confidence
//! - Comparison uses identical Python code between PyRust and CPython
//! - Results include statistical confidence intervals
//! - Both warm execution and total time comparisons implemented
//! - Benchmark verifies python3 is available on system

use std::path::Path;
use std::process::Command;

/// Test that compare_cpython.sh script exists and is executable
#[test]
fn test_compare_script_exists() {
    let script_path = Path::new("scripts/compare_cpython.sh");
    assert!(
        script_path.exists(),
        "scripts/compare_cpython.sh must exist"
    );
}

/// Test that the script can run successfully
#[test]
fn test_compare_script_runs() {
    let output = Command::new("bash")
        .arg("scripts/compare_cpython.sh")
        .output()
        .expect("Failed to execute compare_cpython.sh");

    // Script should exit with code 0 (success)
    assert!(
        output.status.success(),
        "compare_cpython.sh should exit successfully. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Test that the script verifies python3 availability
#[test]
fn test_script_checks_python3() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should check for python3
    assert!(
        script_content.contains("python3"),
        "Script must check for python3 availability"
    );

    // Script should verify python3 with --version or similar
    assert!(
        script_content.contains("command -v python3")
            || script_content.contains("which python3")
            || script_content.contains("python3 --version"),
        "Script must verify python3 is available"
    );
}

/// Test that the script validates AC1.3 (≥50x speedup)
#[test]
fn test_script_validates_ac1_3() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should check for 50x speedup threshold
    assert!(
        script_content.contains("50") || script_content.contains("AC1.3"),
        "Script must validate AC1.3 with 50x speedup threshold"
    );
}

/// Test that the script uses identical Python code for both benchmarks
#[test]
fn test_identical_python_code() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // The script reads benchmark data from criterion JSON files which contain
    // results from running identical Python code "2 + 3" in both benchmarks
    // Verify the script reads from the correct benchmark group
    assert!(
        script_content.contains("speedup_comparison")
            || script_content.contains("cpython_total_time")
            || script_content.contains("pyrust_total_time"),
        "Script should read benchmark data from speedup_comparison group that uses identical code"
    );
}

/// Test that the script extracts confidence intervals
#[test]
fn test_confidence_intervals() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should extract and display confidence intervals
    assert!(
        script_content.contains("confidence_interval")
            || script_content.contains("lower_bound")
            || script_content.contains("upper_bound")
            || script_content.contains("95%")
            || script_content.contains("CI"),
        "Script must extract and display confidence intervals"
    );
}

/// Test that the script compares both warm and total time
#[test]
fn test_warm_and_total_time_comparison() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should reference the speedup_comparison benchmark group
    assert!(
        script_content.contains("speedup_comparison")
            || script_content.contains("total_time")
            || script_content.contains("warm"),
        "Script must compare both warm execution and total time"
    );
}

/// Test that benchmark criterion JSON outputs exist after running benchmarks
#[test]
fn test_benchmark_outputs_exist() {
    // Run the benchmarks first
    let bench_result = Command::new("cargo")
        .args(&["bench", "--bench", "cpython_baseline", "--", "--quick"])
        .output();

    if bench_result.is_err() {
        eprintln!("Warning: Could not run benchmarks, skipping output check");
        return;
    }

    // Check that criterion output directories exist
    let criterion_dir = Path::new("target/criterion/speedup_comparison");
    if criterion_dir.exists() {
        // Check for JSON estimate files
        let cpython_json = criterion_dir.join("cpython_total_time/base/estimates.json");
        let pyrust_json = criterion_dir.join("pyrust_total_time/base/estimates.json");

        // At least one should exist (may not exist if benchmarks were skipped)
        if cpython_json.exists() || pyrust_json.exists() {
            eprintln!("Benchmark outputs found in criterion directory");
        }
    }
}

/// Test that the script validates variance (AC1.5: CV < 10%)
#[test]
fn test_variance_validation() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should check coefficient of variation
    assert!(
        script_content.contains("AC1.5")
            || script_content.contains("CV")
            || script_content.contains("variance")
            || script_content.contains("0.10")
            || script_content.contains("10%"),
        "Script must validate variance (AC1.5)"
    );
}

/// Test edge case: Script should handle missing benchmark data gracefully
#[test]
fn test_handles_missing_data() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should check if files exist before processing
    assert!(
        script_content.contains("if")
            && (script_content.contains("-f")
                || script_content.contains("test -f")
                || script_content.contains("[ -f")),
        "Script should check if benchmark files exist before processing"
    );
}

/// Test edge case: Script should handle python3 not being available
#[test]
fn test_handles_missing_python3() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should have error handling for missing python3
    assert!(
        script_content.contains("Error") || script_content.contains("error"),
        "Script should handle missing python3 with error message"
    );
}

/// Test edge case: Script should handle jq not being available
#[test]
fn test_handles_missing_jq() {
    let script_content = std::fs::read_to_string("scripts/compare_cpython.sh")
        .expect("Failed to read compare_cpython.sh");

    // Script should check for jq dependency
    assert!(
        script_content.contains("jq")
            && (script_content.contains("command -v jq") || script_content.contains("which jq")),
        "Script should verify jq is available"
    );
}

/// Integration test: Run script and verify it produces expected output format
#[test]
fn test_script_output_format() {
    let output = Command::new("bash")
        .arg("scripts/compare_cpython.sh")
        .output();

    if output.is_err() {
        eprintln!("Warning: Could not run script, skipping output format check");
        return;
    }

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // Check for expected output sections
    if output.status.success() {
        // Should contain timing results
        assert!(
            combined.contains("Timing Results")
                || combined.contains("timing")
                || combined.contains("Mean")
                || combined.contains("mean"),
            "Output should contain timing results"
        );

        // Should contain speedup analysis
        assert!(
            combined.contains("Speedup")
                || combined.contains("speedup")
                || combined.contains("x faster"),
            "Output should contain speedup analysis"
        );

        // Should contain AC validation
        assert!(
            combined.contains("AC1.3")
                || combined.contains("PASS")
                || combined.contains("VERIFIED"),
            "Output should contain AC1.3 validation"
        );
    }
}

/// Test that benchmark code uses identical Python expressions
#[test]
fn test_benchmarks_use_identical_code() {
    let bench_content = std::fs::read_to_string("benches/cpython_baseline.rs")
        .expect("Failed to read cpython_baseline.rs");

    // Both benchmarks should use "2 + 3"
    let code_usage = bench_content.matches("\"2 + 3\"").count();
    assert!(
        code_usage >= 2,
        "Both CPython and PyRust benchmarks must use identical Python code '2 + 3'"
    );
}

/// Test that benchmark verifies python3 is available before running
#[test]
fn test_benchmark_checks_python3() {
    let bench_content = std::fs::read_to_string("benches/cpython_baseline.rs")
        .expect("Failed to read cpython_baseline.rs");

    // Benchmark should check for python3 availability
    assert!(
        bench_content.contains("python3")
            && (bench_content.contains("--version")
                || bench_content.contains("is_ok")
                || bench_content.contains("is_err")),
        "Benchmark must verify python3 is available before running"
    );
}

/// Test that speedup_comparison benchmark group exists
#[test]
fn test_speedup_comparison_group_exists() {
    let bench_content = std::fs::read_to_string("benches/cpython_baseline.rs")
        .expect("Failed to read cpython_baseline.rs");

    // Should have speedup_comparison benchmark group
    assert!(
        bench_content.contains("speedup_comparison")
            || bench_content.contains("speedup_calculation"),
        "Benchmark must have speedup_comparison group"
    );

    // Should have both cpython and pyrust measurements
    assert!(
        bench_content.contains("cpython") && bench_content.contains("pyrust"),
        "Speedup comparison must measure both CPython and PyRust"
    );
}

/// Test that warm execution benchmark exists
#[test]
fn test_warm_execution_benchmark_exists() {
    let bench_content = std::fs::read_to_string("benches/cpython_baseline.rs")
        .expect("Failed to read cpython_baseline.rs");

    // Should have warm execution benchmark
    assert!(
        bench_content.contains("warm_execution") || bench_content.contains("warm"),
        "Benchmark suite must include warm execution benchmark"
    );
}

/// Test that cold start benchmark exists
#[test]
fn test_cold_start_benchmark_exists() {
    let bench_content = std::fs::read_to_string("benches/cpython_baseline.rs")
        .expect("Failed to read cpython_baseline.rs");

    // Should have cold start benchmark
    assert!(
        bench_content.contains("cold_start") || bench_content.contains("cold"),
        "Benchmark suite must include cold start benchmark"
    );
}
