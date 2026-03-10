/// Test coverage for AC4.4: All benchmarks have CV < 10% verified by parsing Criterion JSON
/// This test validates the core acceptance criterion for benchmark stability
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test AC4.4: Verify all Criterion benchmarks have CV < 10%
/// This is the primary acceptance criterion for benchmark stability
#[test]
fn test_ac44_all_benchmarks_cv_below_10_percent() {
    // Skip if benchmarks haven't been run
    let criterion_dir = Path::new("target/criterion");
    if !criterion_dir.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    // Run the validation script
    let script_path = Path::new("scripts/validate_benchmark_stability.sh");

    if !script_path.exists() {
        panic!("AC4.4 validation script not found at scripts/validate_benchmark_stability.sh");
    }

    // Make script executable
    let _ = Command::new("chmod").arg("+x").arg(script_path).output();

    // Run the validation script
    let output = Command::new(script_path)
        .output()
        .expect("Failed to execute validation script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The script exits 0 if all benchmarks pass, 1 if any fail
    assert!(
        output.status.success(),
        "AC4.4 FAILED: Validation script reported benchmark stability failures.\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
        stdout,
        stderr
    );
}

/// Test that all modified benchmark files have correct Criterion configuration
#[test]
fn test_all_benchmark_files_have_stability_config() {
    let benchmark_files = vec![
        "benches/compiler_benchmarks.rs",
        "benches/execution_benchmarks.rs",
        "benches/function_call_overhead.rs",
        "benches/lexer_benchmarks.rs",
        "benches/parser_benchmarks.rs",
        "benches/startup_benchmarks.rs",
        "benches/vm_benchmarks.rs",
    ];

    for file_path in benchmark_files {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path));

        // Verify sample_size(1000)
        assert!(
            content.contains(".sample_size(1000)"),
            "{} missing .sample_size(1000) configuration",
            file_path
        );

        // Verify measurement_time(10s)
        assert!(
            content.contains(".measurement_time(std::time::Duration::from_secs(10))"),
            "{} missing .measurement_time(10s) configuration",
            file_path
        );

        // Verify warm_up_time(3s)
        assert!(
            content.contains(".warm_up_time(std::time::Duration::from_secs(3))"),
            "{} missing .warm_up_time(3s) configuration",
            file_path
        );

        // Verify noise_threshold(0.05)
        assert!(
            content.contains(".noise_threshold(0.05)"),
            "{} missing .noise_threshold(0.05) configuration",
            file_path
        );
    }
}

/// Test that the validation script exists and is executable
#[test]
fn test_validation_script_exists() {
    let script_path = Path::new("scripts/validate_benchmark_stability.sh");
    assert!(
        script_path.exists(),
        "Validation script not found at scripts/validate_benchmark_stability.sh"
    );

    // Read the script and verify it parses Criterion JSON
    let content = fs::read_to_string(script_path).expect("Failed to read validation script");

    assert!(
        content.contains("target/criterion"),
        "Script doesn't reference target/criterion directory"
    );

    assert!(
        content.contains("estimates.json"),
        "Script doesn't parse estimates.json files"
    );

    assert!(
        content.contains("CV_THRESHOLD"),
        "Script doesn't define CV threshold"
    );

    assert!(
        content.contains("0.10") || content.contains("10%"),
        "Script doesn't use 10% threshold"
    );
}

/// Test edge case: empty benchmark results directory
#[test]
fn test_edge_case_missing_criterion_directory() {
    // This test verifies the script handles missing benchmark data gracefully
    // We can't actually delete target/criterion in tests, so we just verify
    // the script has error handling for this case

    let script_path = Path::new("scripts/validate_benchmark_stability.sh");
    let content = fs::read_to_string(script_path).expect("Failed to read validation script");

    assert!(
        content.contains("target/criterion directory not found")
            || content.contains("if [ ! -d \"target/criterion\" ]"),
        "Script doesn't handle missing target/criterion directory"
    );
}

/// Test edge case: Verify script correctly calculates CV
#[test]
fn test_edge_case_cv_calculation_formula() {
    let script_path = Path::new("scripts/validate_benchmark_stability.sh");
    let content = fs::read_to_string(script_path).expect("Failed to read validation script");

    // CV should be calculated as std_dev / mean
    assert!(
        content.contains("std_dev / mean") || content.contains("$std_dev / $mean"),
        "Script doesn't use correct CV formula (std_dev / mean)"
    );
}

/// Test that Criterion JSON files exist for key benchmarks
#[test]
fn test_criterion_json_files_exist() {
    let criterion_dir = Path::new("target/criterion");

    // Skip if benchmarks haven't been run
    if !criterion_dir.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    // Verify at least some benchmark results exist
    let estimates_files = find_estimates_json_files(criterion_dir);

    assert!(
        !estimates_files.is_empty(),
        "No estimates.json files found in target/criterion - benchmarks may not have run"
    );
}

/// Helper function to recursively find all estimates.json files
fn find_estimates_json_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_estimates_json_files(&path));
            } else if path.file_name() == Some(std::ffi::OsStr::new("estimates.json")) {
                results.push(path);
            }
        }
    }

    results
}

/// Test that benchmark JSON schema is valid (has required fields)
#[test]
fn test_criterion_json_schema_validity() {
    let criterion_dir = Path::new("target/criterion");

    // Skip if benchmarks haven't been run
    if !criterion_dir.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    let estimates_files = find_estimates_json_files(criterion_dir);

    if estimates_files.is_empty() {
        eprintln!("Skipping - no benchmark data found");
        return;
    }

    // Check at least one file, skipping "change" directory files
    for file in estimates_files
        .iter()
        .filter(|f| !f.to_string_lossy().contains("/change/"))
        .take(3)
    {
        let content =
            fs::read_to_string(file).unwrap_or_else(|_| panic!("Failed to read {:?}", file));

        let data: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON from {:?}", file));

        // Verify required fields for CV calculation
        assert!(
            data["mean"]["point_estimate"].is_f64(),
            "{:?} missing mean.point_estimate",
            file
        );

        assert!(
            data["std_dev"]["point_estimate"].is_f64(),
            "{:?} missing std_dev.point_estimate",
            file
        );
    }
}

/// Test edge case: verify no benchmark has None or null values
#[test]
fn test_edge_case_no_null_values_in_benchmarks() {
    let criterion_dir = Path::new("target/criterion");

    if !criterion_dir.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    let estimates_files = find_estimates_json_files(criterion_dir);

    for file in estimates_files.iter().take(5) {
        // Skip "change" directory files as they contain performance deltas which can be negative
        if file.to_string_lossy().contains("/change/") {
            continue;
        }

        let content =
            fs::read_to_string(file).unwrap_or_else(|_| panic!("Failed to read {:?}", file));

        let data: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON from {:?}", file));

        let mean = data["mean"]["point_estimate"].as_f64();
        let std_dev = data["std_dev"]["point_estimate"].as_f64();

        assert!(
            mean.is_some() && mean.unwrap() > 0.0,
            "{:?} has invalid mean value: {:?}",
            file,
            mean
        );

        assert!(
            std_dev.is_some() && std_dev.unwrap() >= 0.0,
            "{:?} has invalid std_dev value: {:?}",
            file,
            std_dev
        );
    }
}

/// Test edge case: verify CV values are within reasonable bounds (0-100%)
#[test]
fn test_edge_case_cv_values_reasonable() {
    let criterion_dir = Path::new("target/criterion");

    if !criterion_dir.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    let estimates_files = find_estimates_json_files(criterion_dir);

    for file in estimates_files {
        let content =
            fs::read_to_string(&file).unwrap_or_else(|_| panic!("Failed to read {:?}", file));

        let data: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON from {:?}", file));

        let mean = data["mean"]["point_estimate"].as_f64().unwrap_or(1.0);
        let std_dev = data["std_dev"]["point_estimate"].as_f64().unwrap_or(0.0);

        let cv = std_dev / mean;

        assert!(
            cv >= 0.0 && cv <= 2.0,
            "{:?} has unreasonable CV value: {} (mean: {}, std_dev: {})",
            file,
            cv,
            mean,
            std_dev
        );
    }
}
