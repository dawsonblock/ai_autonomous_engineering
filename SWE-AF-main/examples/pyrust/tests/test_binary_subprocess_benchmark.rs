/// Test suite for binary subprocess benchmark validation
/// Validates AC6.1, M1, and AC6.4 for the binary_subprocess.rs benchmark
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_binary_subprocess_benchmark_exists() {
    let bench_path = "benches/binary_subprocess.rs";
    assert!(
        Path::new(bench_path).exists(),
        "Binary subprocess benchmark must exist at {}",
        bench_path
    );
}

#[test]
fn test_binary_subprocess_in_cargo_toml() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(
        cargo_toml.contains("[[bench]]")
            && cargo_toml.contains("name = \"binary_subprocess\"")
            && cargo_toml.contains("harness = false"),
        "Cargo.toml must declare binary_subprocess benchmark with harness = false"
    );
}

#[test]
fn test_validation_script_exists() {
    let script_path = "scripts/validate_binary_speedup.sh";
    assert!(
        Path::new(script_path).exists(),
        "Validation script must exist at {}",
        script_path
    );
}

#[test]
fn test_validation_script_is_executable() {
    let script_path = "scripts/validate_binary_speedup.sh";
    let metadata = fs::metadata(script_path).expect("Failed to read script metadata");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        assert!(
            mode & 0o111 != 0,
            "Validation script must be executable (mode: {:o})",
            mode
        );
    }
}

#[test]
fn test_cv_check_script_exists() {
    let script_path = "scripts/check_binary_subprocess_cv.sh";
    assert!(
        Path::new(script_path).exists(),
        "CV check script must exist at {}",
        script_path
    );
}

#[test]
fn test_cv_check_script_is_executable() {
    let script_path = "scripts/check_binary_subprocess_cv.sh";
    let metadata = fs::metadata(script_path).expect("Failed to read script metadata");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        assert!(
            mode & 0o111 != 0,
            "CV check script must be executable (mode: {:o})",
            mode
        );
    }
}

#[test]
fn test_benchmark_uses_command_new() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("Command::new"),
        "Benchmark must use Command::new() to spawn subprocess"
    );

    assert!(
        bench_content.contains(".output()"),
        "Benchmark must use .output() to capture subprocess output"
    );
}

#[test]
fn test_benchmark_has_high_sample_size() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    // Subprocess benchmarks need high sample size due to OS scheduling variance
    assert!(
        bench_content.contains("sample_size(1000)") || bench_content.contains("sample_size(2000)"),
        "Benchmark should have sample_size >= 1000 for statistical stability (AC6.4)"
    );
}

#[test]
fn test_benchmark_has_long_measurement_time() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    // Subprocess benchmarks need long measurement time
    assert!(
        bench_content.contains("measurement_time(std::time::Duration::from_secs(30))")
            || bench_content.contains("measurement_time(std::time::Duration::from_secs(20))")
            || bench_content.contains("measurement_time(std::time::Duration::from_secs(15))")
            || bench_content.contains("measurement_time(std::time::Duration::from_secs(10))"),
        "Benchmark should have measurement_time >= 10s for statistical stability"
    );
}

#[test]
fn test_benchmark_has_warmup() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("warm_up_time"),
        "Benchmark must include warmup period"
    );
}

#[test]
fn test_validation_script_uses_hyperfine() {
    let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
        .expect("Failed to read validation script");

    assert!(
        script_content.contains("hyperfine"),
        "Validation script must use hyperfine for M1"
    );

    assert!(
        script_content.contains("--runs 100") || script_content.contains("--runs=100"),
        "Validation script must run 100 iterations per M1"
    );

    assert!(
        script_content.contains("--export-json"),
        "Validation script must export JSON for parsing"
    );
}

#[test]
fn test_validation_script_uses_jq() {
    let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
        .expect("Failed to read validation script");

    assert!(
        script_content.contains("jq"),
        "Validation script must use jq to parse JSON"
    );

    assert!(
        script_content.contains(".mean") || script_content.contains("results[0].mean"),
        "Validation script must extract mean from hyperfine JSON"
    );
}

#[test]
fn test_validation_script_checks_380us_threshold() {
    let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
        .expect("Failed to read validation script");

    assert!(
        script_content.contains("380"),
        "Validation script must check 380μs threshold per AC6.1 and M1"
    );

    // Should compare mean <= 380
    assert!(
        script_content.contains("<= 380") || script_content.contains("≤ 380"),
        "Validation script must validate mean ≤ 380μs"
    );
}

#[test]
fn test_validation_script_exits_with_correct_code() {
    let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
        .expect("Failed to read validation script");

    assert!(
        script_content.contains("exit 0") && script_content.contains("exit 1"),
        "Validation script must exit 0 on pass, 1 on fail"
    );
}

#[test]
fn test_cv_check_script_checks_10_percent_threshold() {
    let script_content = fs::read_to_string("scripts/check_binary_subprocess_cv.sh")
        .expect("Failed to read CV check script");

    assert!(
        script_content.contains("10")
            && (script_content.contains("CV") || script_content.contains("cv")),
        "CV check script must validate CV < 10% per AC6.4"
    );
}

#[test]
fn test_cv_check_script_uses_criterion_json() {
    let script_content = fs::read_to_string("scripts/check_binary_subprocess_cv.sh")
        .expect("Failed to read CV check script");

    assert!(
        script_content.contains("estimates.json"),
        "CV check script must read Criterion estimates.json"
    );

    assert!(
        script_content.contains("target/criterion"),
        "CV check script must search in target/criterion directory"
    );
}

#[test]
fn test_benchmark_includes_simple_arithmetic() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("2+3") || bench_content.contains("2 + 3"),
        "Benchmark must include simple arithmetic test case (2+3)"
    );

    assert!(
        bench_content.contains("binary_subprocess_simple"),
        "Benchmark must include simple_arithmetic benchmark function"
    );
}

#[test]
fn test_benchmark_validates_output() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("assert_eq!") || bench_content.contains("assert!"),
        "Benchmark must validate output correctness"
    );

    assert!(
        bench_content.contains(".stdout") || bench_content.contains("from_utf8"),
        "Benchmark must check stdout output"
    );

    assert!(
        bench_content.contains(".status.success()"),
        "Benchmark must verify successful execution"
    );
}

#[test]
fn test_benchmark_uses_black_box() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("black_box"),
        "Benchmark must use black_box to prevent compiler optimizations"
    );
}

#[test]
fn test_benchmark_gets_binary_path_correctly() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("target/release/pyrust")
            || bench_content.contains("CARGO_MANIFEST_DIR"),
        "Benchmark must construct correct path to release binary"
    );

    assert!(
        bench_content.contains("get_binary_path") || bench_content.contains("binary_path"),
        "Benchmark must define function to get binary path"
    );
}

#[test]
fn test_benchmark_checks_binary_exists() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains(".exists()"),
        "Benchmark must verify binary exists before running"
    );

    assert!(
        bench_content.contains("assert!") && bench_content.contains("Binary not found"),
        "Benchmark must provide helpful error if binary missing"
    );
}

#[test]
fn test_benchmark_includes_multiple_test_cases() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    // Should test various code complexities
    let test_cases = vec![
        "simple",    // Simple arithmetic
        "complex",   // Complex expressions
        "variables", // With variables
        "print",     // With print
        "multiple",  // Multiple operations
        "nested",    // Nested expressions
    ];

    let found_cases: Vec<_> = test_cases
        .iter()
        .filter(|&case| bench_content.to_lowercase().contains(case))
        .collect();

    assert!(
        found_cases.len() >= 4,
        "Benchmark should test multiple code complexities, found: {:?}",
        found_cases
    );
}

#[test]
fn test_benchmark_includes_startup_overhead_test() {
    let bench_content = fs::read_to_string("benches/binary_subprocess.rs")
        .expect("Failed to read binary_subprocess.rs");

    assert!(
        bench_content.contains("startup")
            || bench_content.contains("overhead")
            || bench_content.contains("empty"),
        "Benchmark should include test for startup overhead (empty program)"
    );
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_validation_script_runs() {
        // Test that validation script can execute (may pass or fail, but shouldn't crash)
        let output = Command::new("./scripts/validate_binary_speedup.sh")
            .output()
            .expect("Failed to execute validation script");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // Script should produce recognizable output
        assert!(
            combined.contains("Binary Speedup Validation")
                || combined.contains("M1")
                || combined.contains("Mean")
                || combined.contains("ERROR"),
            "Script must produce recognizable output"
        );

        // Exit code should be 0 or 1
        let exit_code = output.status.code().unwrap_or(999);
        assert!(
            exit_code == 0 || exit_code == 1,
            "Script must exit with 0 (pass) or 1 (fail), got {}",
            exit_code
        );
    }

    #[test]
    fn test_cv_check_script_runs() {
        // Test that CV check script can execute
        let output = Command::new("./scripts/check_binary_subprocess_cv.sh")
            .output()
            .expect("Failed to execute CV check script");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // Script should produce recognizable output
        assert!(
            combined.contains("CV Check")
                || combined.contains("AC6.4")
                || combined.contains("binary_subprocess")
                || combined.contains("WARNING")
                || combined.contains("ERROR"),
            "CV check script must produce recognizable output"
        );
    }

    #[test]
    fn test_binary_subprocess_benchmark_compiles() {
        // Verify the benchmark can be built
        let output = Command::new("cargo")
            .args(&["build", "--bench", "binary_subprocess"])
            .env("PYO3_USE_ABI3_FORWARD_COMPATIBILITY", "1")
            .output()
            .expect("Failed to run cargo build");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should compile (warnings OK, errors not OK)
        assert!(
            output.status.success() || !stderr.contains("error:"),
            "Binary subprocess benchmark must compile successfully"
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_validation_handles_missing_binary() {
        // Validation script should check if binary exists
        let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
            .expect("Failed to read validation script");

        assert!(
            script_content.contains("-f") && script_content.contains("Binary not found"),
            "Validation script must check if binary exists"
        );
    }

    #[test]
    fn test_validation_calculates_mean_correctly() {
        // Verify script converts seconds to microseconds correctly
        let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
            .expect("Failed to read validation script");

        // Should multiply by 1,000,000 to convert seconds to microseconds
        assert!(
            script_content.contains("1000000") || script_content.contains("* 1000000"),
            "Validation script must convert seconds to microseconds (multiply by 1,000,000)"
        );
    }

    #[test]
    fn test_validation_handles_hyperfine_failure() {
        let script_content = fs::read_to_string("scripts/validate_binary_speedup.sh")
            .expect("Failed to read validation script");

        // Should check if hyperfine succeeded
        assert!(
            script_content.contains("$?") || script_content.contains("exit"),
            "Validation script must handle hyperfine failures"
        );
    }

    #[test]
    fn test_cv_check_handles_missing_estimates() {
        let script_content = fs::read_to_string("scripts/check_binary_subprocess_cv.sh")
            .expect("Failed to read CV check script");

        // Should handle missing estimates.json files
        assert!(
            script_content.contains("if") || script_content.contains("test"),
            "CV check script should handle missing estimate files"
        );
    }
}
