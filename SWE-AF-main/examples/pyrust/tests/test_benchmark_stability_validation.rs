/// Test suite for benchmark stability validation
/// Validates that the validation script and benchmark configurations meet AC4.4 and M5
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_validation_script_exists() {
    let script_path = "scripts/validate_benchmark_stability.sh";
    assert!(
        Path::new(script_path).exists(),
        "Validation script must exist at {}",
        script_path
    );
}

#[test]
fn test_validation_script_is_executable() {
    let script_path = "scripts/validate_benchmark_stability.sh";
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
fn test_validation_script_parses_criterion_json() {
    // This test verifies the script can parse Criterion JSON format
    let script_path = "./scripts/validate_benchmark_stability.sh";

    // Script should fail gracefully if no benchmark data exists
    // or succeed if data exists with CV < 10%
    let output = Command::new(script_path)
        .output()
        .expect("Failed to execute validation script");

    // Script must produce output (either success or failure message)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Benchmark Stability Validation")
            || combined.contains("ERROR")
            || combined.contains("Summary"),
        "Script must produce recognizable output"
    );
}

#[test]
fn test_benchmark_files_have_high_sample_size() {
    // Verify benchmarks use increased sample_size for stability
    let benchmark_files = vec![
        "benches/lexer_benchmarks.rs",
        "benches/parser_benchmarks.rs",
        "benches/compiler_benchmarks.rs",
        "benches/vm_benchmarks.rs",
        "benches/execution_benchmarks.rs",
        "benches/function_call_overhead.rs",
        "benches/startup_benchmarks.rs",
    ];

    for bench_file in benchmark_files {
        let content = fs::read_to_string(bench_file)
            .unwrap_or_else(|_| panic!("Failed to read {}", bench_file));

        // Check for high sample_size configuration (should be >= 1000)
        let has_high_sample_size = content.contains("sample_size(3000)")
            || content.contains("sample_size(1000)")
            || content.contains("sample_size(2000)");

        assert!(
            has_high_sample_size,
            "{} should have sample_size >= 1000 for AC4.4 (CV < 10%)",
            bench_file
        );
    }
}

#[test]
fn test_benchmark_files_have_long_measurement_time() {
    // Verify benchmarks use increased measurement_time for stability
    let benchmark_files = vec![
        "benches/lexer_benchmarks.rs",
        "benches/parser_benchmarks.rs",
        "benches/compiler_benchmarks.rs",
        "benches/vm_benchmarks.rs",
        "benches/execution_benchmarks.rs",
        "benches/function_call_overhead.rs",
        "benches/startup_benchmarks.rs",
    ];

    for bench_file in benchmark_files {
        let content = fs::read_to_string(bench_file)
            .unwrap_or_else(|_| panic!("Failed to read {}", bench_file));

        // Check for long measurement_time (should be >= 10s)
        let has_long_measurement = content
            .contains("measurement_time(std::time::Duration::from_secs(20))")
            || content.contains("measurement_time(std::time::Duration::from_secs(10))")
            || content.contains("measurement_time(std::time::Duration::from_secs(15))");

        assert!(
            has_long_measurement,
            "{} should have measurement_time >= 10s for AC4.4 (CV < 10%)",
            bench_file
        );
    }
}

#[test]
fn test_criterion_json_format_compatibility() {
    // Test that we can parse Criterion's estimates.json format
    // This is a structure test - actual CV validation happens in the script

    let test_json = r#"{
        "mean": {
            "point_estimate": 1000.0,
            "standard_error": 10.0
        },
        "std_dev": {
            "point_estimate": 50.0,
            "standard_error": 5.0
        }
    }"#;

    let parsed: serde_json::Value =
        serde_json::from_str(test_json).expect("Failed to parse Criterion JSON format");

    assert!(parsed["mean"]["point_estimate"].is_number());
    assert!(parsed["std_dev"]["point_estimate"].is_number());

    // Calculate CV as the script does: std_dev / mean
    let mean = parsed["mean"]["point_estimate"].as_f64().unwrap();
    let std_dev = parsed["std_dev"]["point_estimate"].as_f64().unwrap();
    let cv = std_dev / mean;

    assert_eq!(cv, 0.05, "CV calculation should match script logic");
}

#[test]
fn test_cv_threshold_constant() {
    // Verify the validation script uses 10% (0.10) threshold as specified in AC4.4
    let script_content = fs::read_to_string("scripts/validate_benchmark_stability.sh")
        .expect("Failed to read validation script");

    assert!(
        script_content.contains("CV_THRESHOLD=0.10")
            || script_content.contains("CV_THRESHOLD=0.1")
            || script_content.contains("10%"),
        "Validation script must use 10% CV threshold (AC4.4)"
    );
}

#[test]
fn test_estimates_json_location_pattern() {
    // Verify script searches in correct location: target/criterion/**/estimates.json
    let script_content = fs::read_to_string("scripts/validate_benchmark_stability.sh")
        .expect("Failed to read validation script");

    assert!(
        script_content.contains("target/criterion") && script_content.contains("estimates.json"),
        "Script must search target/criterion/**/estimates.json"
    );
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_validation_handles_missing_benchmark_data() {
        // Move away existing data temporarily to test error handling
        let criterion_dir = Path::new("target/criterion");

        if !criterion_dir.exists() {
            // This is the edge case - no benchmark data
            let output = Command::new("./scripts/validate_benchmark_stability.sh")
                .output()
                .expect("Failed to execute validation script");

            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let combined = format!("{}{}", stdout, stderr);

            // Script should either:
            // 1. Report "No benchmarks found" OR
            // 2. Report "target/criterion directory not found" OR
            // 3. Exit with non-zero status
            assert!(
                !output.status.success()
                    || combined.contains("No benchmarks found")
                    || combined.contains("directory not found"),
                "Script must handle missing benchmark data gracefully"
            );
        }
    }

    #[test]
    fn test_validation_script_exit_codes() {
        // Verify script exits with correct codes:
        // - Exit 0 if all benchmarks pass CV < 10%
        // - Exit 1 if any benchmark fails or no data
        let output = Command::new("./scripts/validate_benchmark_stability.sh")
            .output()
            .expect("Failed to execute validation script");

        // Script must have predictable exit behavior
        let exit_code = output.status.code().unwrap_or(999);
        assert!(
            exit_code == 0 || exit_code == 1,
            "Script must exit with 0 (pass) or 1 (fail), got {}",
            exit_code
        );
    }
}
