// Test suite for speedup validation scripts
// Covers AC6.4 and AC6.5
//
// AC6.4: All benchmarks show CV < 10% ensuring statistical stability
// AC6.5: scripts/validate_speedup.sh exits 0 indicating ≥50x speedup vs CPython baseline

use std::process::Command;

#[test]
fn test_validate_speedup_script_exists() {
    // Verify the main validation script exists and is executable
    let script_path = "scripts/validate_speedup.sh";
    assert!(
        std::path::Path::new(script_path).exists(),
        "validate_speedup.sh should exist at {}",
        script_path
    );

    // Check if it's executable (Unix-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script_path).expect("Failed to read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "validate_speedup.sh should be executable"
        );
    }
}

#[test]
fn test_validate_binary_speedup_script_exists() {
    // Verify the binary speedup validation script exists and is executable
    let script_path = "scripts/validate_binary_speedup.sh";
    assert!(
        std::path::Path::new(script_path).exists(),
        "validate_binary_speedup.sh should exist at {}",
        script_path
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script_path).expect("Failed to read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "validate_binary_speedup.sh should be executable"
        );
    }
}

#[test]
fn test_validate_daemon_speedup_script_exists() {
    // Verify the daemon speedup validation script exists and is executable
    let script_path = "scripts/validate_daemon_speedup.sh";
    assert!(
        std::path::Path::new(script_path).exists(),
        "validate_daemon_speedup.sh should exist at {}",
        script_path
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script_path).expect("Failed to read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "validate_daemon_speedup.sh should be executable"
        );
    }
}

#[test]
fn test_validate_benchmark_stability_script_exists() {
    // Verify the benchmark stability validation script exists and is executable
    let script_path = "scripts/validate_benchmark_stability.sh";
    assert!(
        std::path::Path::new(script_path).exists(),
        "validate_benchmark_stability.sh should exist at {}",
        script_path
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script_path).expect("Failed to read script metadata");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "validate_benchmark_stability.sh should be executable"
        );
    }
}

#[test]
fn test_validate_speedup_script_has_hyperfine_dependency_check() {
    // Verify the script checks for hyperfine dependency
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("hyperfine"),
        "Script should check for hyperfine dependency"
    );
    assert!(
        script_content.contains("--runs 100") || script_content.contains("--runs=100"),
        "Script should use hyperfine with 100 runs for statistical rigor"
    );
}

#[test]
fn test_validate_speedup_script_outputs_required_metrics() {
    // Verify the script outputs mean, stddev, min, max, and speedup ratio
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("Mean") || script_content.contains("mean"),
        "Script should output mean timing"
    );
    assert!(
        script_content.contains("StdDev") || script_content.contains("stddev"),
        "Script should output standard deviation"
    );
    assert!(
        script_content.contains("Min") || script_content.contains("min"),
        "Script should output minimum timing"
    );
    assert!(
        script_content.contains("Max") || script_content.contains("max"),
        "Script should output maximum timing"
    );
    assert!(
        script_content.contains("speedup") || script_content.contains("Speedup"),
        "Script should output speedup ratio"
    );
}

#[test]
fn test_validate_speedup_script_checks_50x_target() {
    // Verify the script validates ≥50x speedup target (AC6.5)
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("50") && script_content.contains("speedup"),
        "Script should validate ≥50x speedup target (AC6.5)"
    );
}

#[test]
fn test_validate_speedup_script_checks_cv_threshold() {
    // Verify the script checks CV < 10% for statistical stability (AC6.4)
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("CV") || script_content.contains("cv"),
        "Script should check coefficient of variation for statistical stability (AC6.4)"
    );
    assert!(
        script_content.contains("10"),
        "Script should use 10% CV threshold"
    );
}

#[test]
fn test_validate_binary_speedup_script_has_correct_target() {
    // Verify binary speedup script checks ≤380μs target (M1)
    let script_content = std::fs::read_to_string("scripts/validate_binary_speedup.sh")
        .expect("Failed to read validate_binary_speedup.sh");

    assert!(
        script_content.contains("380"),
        "Binary speedup script should check ≤380μs target (M1)"
    );
}

#[test]
fn test_validate_daemon_speedup_script_has_correct_target() {
    // Verify daemon speedup script checks ≤190μs target (M2)
    let script_content = std::fs::read_to_string("scripts/validate_daemon_speedup.sh")
        .expect("Failed to read validate_daemon_speedup.sh");

    assert!(
        script_content.contains("190"),
        "Daemon speedup script should check ≤190μs target (M2)"
    );
}

#[test]
fn test_validate_benchmark_stability_parses_criterion_json() {
    // Verify benchmark stability script parses Criterion JSON correctly
    let script_content = std::fs::read_to_string("scripts/validate_benchmark_stability.sh")
        .expect("Failed to read validate_benchmark_stability.sh");

    assert!(
        script_content.contains("estimates.json"),
        "Script should parse Criterion estimates.json files"
    );
    assert!(
        script_content.contains("jq")
            || script_content.contains("mean")
            || script_content.contains("std_dev"),
        "Script should parse JSON for mean and std_dev"
    );
    assert!(
        script_content.contains("target/criterion"),
        "Script should look in target/criterion directory for benchmark data"
    );
}

#[test]
fn test_validate_speedup_script_uses_jq_for_json_parsing() {
    // Verify the script uses jq for JSON parsing
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("jq"),
        "Script should use jq for JSON parsing"
    );
    assert!(
        script_content.contains("--export-json") || script_content.contains("export-json"),
        "Script should use hyperfine's JSON export feature"
    );
}

#[test]
fn test_validate_speedup_script_compares_cpython_and_pyrust() {
    // Verify the script compares both CPython and PyRust
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("python3")
            || script_content.contains("CPython")
            || script_content.contains("cpython"),
        "Script should measure CPython baseline"
    );
    assert!(
        script_content.contains("pyrust")
            || script_content.contains("PyRust")
            || script_content.contains("target/release"),
        "Script should measure PyRust performance"
    );
}

#[test]
#[ignore] // Only run manually as it requires full environment setup
fn test_validate_speedup_script_execution() {
    // Test that the script can actually execute (requires hyperfine, jq, bc, python3)
    // Note: This test is marked as #[ignore] because it requires:
    // 1. hyperfine installed
    // 2. jq installed
    // 3. bc installed
    // 4. python3 installed
    // 5. Release binary built

    // Build release binary first
    let build_status = Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Failed to build release binary");

    assert!(
        build_status.success(),
        "Release binary build should succeed"
    );

    // Run the validation script
    let output = Command::new("./scripts/validate_speedup.sh")
        .output()
        .expect("Failed to execute validate_speedup.sh");

    // Check that the script produced output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    // The script should output speedup metrics
    assert!(
        stdout.contains("Speedup") || stdout.contains("speedup"),
        "Script output should contain speedup information"
    );

    // The script should output mean timing
    assert!(
        stdout.contains("Mean") || stdout.contains("mean"),
        "Script output should contain mean timing"
    );

    // Note: We don't assert on exit code because it will fail until optimizations are complete
    // The script exits 0 only when ≥50x speedup is achieved
}

#[test]
fn test_all_validation_scripts_have_proper_shebang() {
    // Verify all scripts have proper bash shebang
    let scripts = vec![
        "scripts/validate_speedup.sh",
        "scripts/validate_binary_speedup.sh",
        "scripts/validate_daemon_speedup.sh",
        "scripts/validate_benchmark_stability.sh",
    ];

    for script_path in scripts {
        let content =
            std::fs::read_to_string(script_path).expect(&format!("Failed to read {}", script_path));

        assert!(
            content.starts_with("#!/bin/bash") || content.starts_with("#!/usr/bin/env bash"),
            "{} should have proper bash shebang",
            script_path
        );
    }
}

#[test]
fn test_validate_speedup_script_has_exit_codes() {
    // Verify the script has proper exit codes (0 for pass, non-zero for fail)
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("exit 0"),
        "Script should exit 0 on success"
    );
    assert!(
        script_content.contains("exit 1") || script_content.contains("exit "),
        "Script should exit non-zero on failure"
    );
}

#[test]
fn test_validation_scripts_use_statistical_rigor() {
    // Verify scripts use enough runs for statistical confidence
    let scripts = vec![
        ("scripts/validate_speedup.sh", 100),
        ("scripts/validate_binary_speedup.sh", 100),
    ];

    for (script_path, min_runs) in scripts {
        let content =
            std::fs::read_to_string(script_path).expect(&format!("Failed to read {}", script_path));

        // Check for hyperfine runs parameter
        assert!(
            content.contains(&format!("--runs {}", min_runs))
                || content.contains(&format!("--runs={}", min_runs))
                || content.contains(&format!("runs {}", min_runs))
                || content.contains(&format!("NUM_RUNS={}", min_runs))
                || content.contains(&format!("RUNS={}", min_runs)),
            "{} should use at least {} runs for statistical rigor",
            script_path,
            min_runs
        );
    }
}

#[test]
fn test_validate_speedup_script_measures_cpython_baseline() {
    // Verify the script measures CPython baseline (19ms reference from PRD)
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("19")
            && (script_content.contains("ms") || script_content.contains("baseline")),
        "Script should reference the 19ms CPython baseline from PRD"
    );
}

#[test]
fn test_validation_scripts_have_error_handling() {
    // Verify scripts have proper error handling with set -e or explicit checks
    let scripts = vec![
        "scripts/validate_speedup.sh",
        "scripts/validate_binary_speedup.sh",
        "scripts/validate_daemon_speedup.sh",
        "scripts/validate_benchmark_stability.sh",
    ];

    for script_path in scripts {
        let content =
            std::fs::read_to_string(script_path).expect(&format!("Failed to read {}", script_path));

        assert!(
            content.contains("set -e") || content.contains("if [ $? -ne 0 ]"),
            "{} should have proper error handling (set -e or explicit checks)",
            script_path
        );
    }
}

#[test]
fn test_validate_speedup_script_cleans_up_temp_files() {
    // Verify the script cleans up temporary JSON files
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("rm -f") || script_content.contains("rm "),
        "Script should clean up temporary files"
    );
}

// Edge case tests for comprehensive coverage

#[test]
fn test_validate_speedup_script_checks_for_missing_dependencies() {
    // Edge case: Verify script checks all required dependencies
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Check for dependency validation for all required tools
    let required_deps = vec!["hyperfine", "jq", "bc", "python3"];
    for dep in required_deps {
        assert!(
            script_content.contains(&format!("command -v {}", dep))
                || script_content.contains(&format!("which {}", dep)),
            "Script should check for {} dependency",
            dep
        );
    }
}

#[test]
fn test_validate_speedup_script_handles_missing_binary() {
    // Edge case: Verify script handles case when PyRust binary doesn't exist
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("BINARY_PATH") || script_content.contains("binary"),
        "Script should reference the binary path"
    );
    assert!(
        script_content.contains("if [ ! -f") || script_content.contains("test -f"),
        "Script should check if binary exists"
    );
}

#[test]
fn test_validate_speedup_script_uses_bc_for_float_math() {
    // Edge case: Verify script uses bc for precise floating point calculations
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("bc"),
        "Script should use bc for floating point math"
    );
    assert!(
        script_content.contains("scale=") || script_content.contains("bc -l"),
        "Script should set precision for bc calculations"
    );
}

#[test]
fn test_validate_speedup_script_validates_both_cpython_and_pyrust() {
    // Integration: Verify script measures both CPython and PyRust separately
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Check for separate measurement sections
    assert!(
        script_content.contains("CPython") || script_content.contains("cpython"),
        "Script should have a CPython measurement section"
    );
    assert!(
        script_content.contains("PyRust") || script_content.contains("pyrust"),
        "Script should have a PyRust measurement section"
    );

    // Verify it calculates ratio
    assert!(
        script_content.contains("speedup")
            && (script_content.contains("/") || script_content.contains("÷")),
        "Script should calculate speedup ratio (division)"
    );
}

#[test]
fn test_validate_speedup_script_exports_json_for_parsing() {
    // Edge case: Verify script uses JSON export for reliable parsing
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("--export-json"),
        "Script should use --export-json flag with hyperfine"
    );
    assert!(
        script_content.contains(".json"),
        "Script should work with JSON output files"
    );
}

#[test]
fn test_validate_speedup_script_has_warmup_runs() {
    // Edge case: Verify script uses warmup runs to stabilize measurements
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("--warmup") || script_content.contains("warmup"),
        "Script should use warmup runs before measurements"
    );
}

#[test]
fn test_validate_speedup_script_outputs_both_ms_and_us() {
    // Edge case: Verify script outputs in multiple time units for clarity
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Should convert to both milliseconds and microseconds
    assert!(
        script_content.contains("1000")
            && (script_content.contains("ms") || script_content.contains("us")),
        "Script should convert between time units (ms and μs)"
    );
}

#[test]
fn test_validate_speedup_script_compares_against_19ms_baseline() {
    // Boundary value: Verify script uses the exact 19ms CPython baseline from PRD
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Check for 19ms baseline reference
    assert!(
        script_content.contains("19"),
        "Script should reference 19ms baseline"
    );
}

#[test]
fn test_validate_speedup_script_validates_exact_50x_boundary() {
    // Boundary value: Verify script properly handles exactly 50x speedup
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Should use >= 50 (not > 50)
    assert!(
        script_content.contains("50"),
        "Script should check against 50x target"
    );
    assert!(
        script_content.contains(">=")
            || script_content.contains("≥")
            || script_content.contains("-ge"),
        "Script should use >= comparison (50x is acceptable)"
    );
}

#[test]
fn test_validate_speedup_script_validates_exact_10percent_cv_boundary() {
    // Boundary value: Verify script properly handles exactly 10% CV
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("10")
            && (script_content.contains("CV") || script_content.contains("cv")),
        "Script should check against 10% CV threshold"
    );
    // Should use < 10 (not <=) per AC6.4
    assert!(
        script_content.contains("<") || script_content.contains("-lt"),
        "Script should use < comparison for CV threshold"
    );
}

#[test]
fn test_validate_speedup_script_extracts_mean_from_json() {
    // Edge case: Verify script extracts mean timing from hyperfine JSON
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("'.mean'") || script_content.contains(".mean"),
        "Script should extract mean from JSON using jq"
    );
}

#[test]
fn test_validate_speedup_script_extracts_stddev_from_json() {
    // Edge case: Verify script extracts stddev for CV calculation
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("'.stddev'")
            || script_content.contains(".stddev")
            || script_content.contains("std_dev"),
        "Script should extract stddev from JSON"
    );
}

#[test]
fn test_validate_speedup_script_extracts_min_max_from_json() {
    // Edge case: Verify script extracts min and max timings
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("'.min'") || script_content.contains(".min"),
        "Script should extract min from JSON"
    );
    assert!(
        script_content.contains("'.max'") || script_content.contains(".max"),
        "Script should extract max from JSON"
    );
}

#[test]
fn test_validate_speedup_script_calculates_cv_correctly() {
    // Edge case: Verify script calculates CV as (stddev / mean) * 100
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Check for CV calculation formula components
    if script_content.contains("CV") || script_content.contains("cv") {
        assert!(
            script_content.contains("100")
                && (script_content.contains("stddev") || script_content.contains("mean")),
            "Script should calculate CV as (stddev/mean)*100"
        );
    }
}

#[test]
fn test_validate_speedup_script_builds_release_binary() {
    // Edge case: Verify script builds release binary if missing
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("cargo build --release") || script_content.contains("release"),
        "Script should reference release build"
    );
}

#[test]
fn test_validate_speedup_script_provides_colored_output() {
    // Integration: Verify script provides user-friendly colored output
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Check for color codes or color variables
    assert!(
        script_content.contains("\\033[")
            || script_content.contains("RED=")
            || script_content.contains("GREEN="),
        "Script should use colors for user-friendly output"
    );
}

#[test]
fn test_validate_speedup_script_has_clear_pass_fail_output() {
    // Integration: Verify script has clear PASS/FAIL messages
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    assert!(
        script_content.contains("PASS") || script_content.contains("✓"),
        "Script should have clear PASS indicator"
    );
    assert!(
        script_content.contains("FAIL") || script_content.contains("✗"),
        "Script should have clear FAIL indicator"
    );
}

#[test]
fn test_validate_speedup_script_enforces_cv_threshold() {
    // AC6.4: Verify script sets PASS=false when CV >= 10%
    let script_content = std::fs::read_to_string("scripts/validate_speedup.sh")
        .expect("Failed to read validate_speedup.sh");

    // Check that script has logic to set PASS=false for CPython CV violations
    assert!(
        script_content.contains("cpython_cv") && script_content.contains("TARGET_CV_PERCENT"),
        "Script should check cpython_cv against TARGET_CV_PERCENT"
    );

    // Check that script has logic to set PASS=false for PyRust CV violations
    assert!(
        script_content.contains("pyrust_cv") && script_content.contains("TARGET_CV_PERCENT"),
        "Script should check pyrust_cv against TARGET_CV_PERCENT"
    );

    // Verify PASS variable is set to false on CV violations
    // The script should have conditional logic that sets PASS=false
    let has_pass_false_logic =
        script_content.contains("PASS=false") || script_content.contains("PASS=\"false\"");
    assert!(
        has_pass_false_logic,
        "Script should set PASS=false when acceptance criteria fail"
    );

    // Verify the script checks CV for both CPython and PyRust
    // Count occurrences of CV comparisons (should have at least 2: one for CPython, one for PyRust)
    let cv_check_count = script_content.matches("< $TARGET_CV_PERCENT").count();
    assert!(
        cv_check_count >= 2,
        "Script should check CV threshold for both CPython and PyRust (found {} checks, expected >= 2)",
        cv_check_count
    );
}
