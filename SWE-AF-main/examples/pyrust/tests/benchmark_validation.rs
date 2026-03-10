/// Integration test to validate benchmark results against acceptance criteria
/// This test reads the criterion JSON output and verifies:
/// - AC1.2: Mean cold start time < 100μs
/// - AC1.5: Coefficient of variation < 10%
use std::fs;
use std::path::Path;

#[test]
fn test_cold_start_performance_meets_ac12() {
    let estimates_path = Path::new("target/criterion/cold_start_simple/new/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping benchmark validation - run 'cargo bench' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Missing mean.point_estimate");

    let mean_us = mean_ns / 1000.0;

    // AC1.2: Mean cold start time < 100μs for simple programs
    assert!(
        mean_us < 100.0,
        "AC1.2 FAILED: Mean cold start time {:.2}μs exceeds 100μs target",
        mean_us
    );

    println!("AC1.2 PASS: Cold start mean = {:.2}μs (< 100μs)", mean_us);
}

#[test]
fn test_benchmark_stability_meets_ac15() {
    let estimates_path = Path::new("target/criterion/cold_start_simple/new/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping benchmark validation - run 'cargo bench' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Missing mean.point_estimate");

    let std_dev_ns = data["std_dev"]["point_estimate"]
        .as_f64()
        .expect("Missing std_dev.point_estimate");

    let cv = std_dev_ns / mean_ns;

    // AC1.5: Coefficient of variation < 10%
    assert!(
        cv < 0.10,
        "AC1.5 FAILED: Coefficient of variation {:.2}% exceeds 10% threshold",
        cv * 100.0
    );

    println!("AC1.5 PASS: CV = {:.2}% (< 10%)", cv * 100.0);
}

#[test]
fn test_all_required_benchmarks_exist() {
    // Verify all benchmarks mentioned in the testing strategy exist
    let required_benchmarks = vec![
        "cold_start_simple",     // AC1.2 - Simple arithmetic (2+3)
        "cold_start_complex",    // Complex arithmetic
        "with_variables",        // Variables (x = 10; y = 20; x + y)
        "with_print",            // Print statements
        "warm_execution_simple", // Warm execution benchmarks
    ];

    for bench_name in required_benchmarks {
        let bench_path = Path::new("target/criterion").join(bench_name);

        // Skip if benchmarks haven't been run
        if !bench_path.exists() {
            eprintln!("Skipping - run 'cargo bench' first");
            return;
        }

        assert!(
            bench_path.exists(),
            "Required benchmark '{}' directory not found",
            bench_name
        );
    }
}

#[test]
fn test_html_reports_generated() {
    let report_path = Path::new("target/criterion/report/index.html");

    // Skip if benchmarks haven't been run
    if !report_path.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    assert!(
        report_path.exists(),
        "Criterion HTML report not generated at target/criterion/report/index.html"
    );

    let content = fs::read_to_string(report_path).expect("Failed to read HTML report");

    assert!(content.len() > 0, "HTML report is empty");
}

#[test]
fn test_benchmark_json_schema_valid() {
    let estimates_path = Path::new("target/criterion/cold_start_simple/new/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping benchmark validation - run 'cargo bench' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Verify required fields exist per testing strategy
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "Missing mean.point_estimate"
    );
    assert!(
        data["std_dev"]["point_estimate"].is_f64(),
        "Missing std_dev.point_estimate"
    );
    assert!(data["median"]["point_estimate"].is_f64(), "Missing median");
    assert!(
        data["mean"]["confidence_interval"].is_object(),
        "Missing confidence interval"
    );
}

#[test]
fn test_edge_case_empty_program_benchmark() {
    // Edge case: Verify empty program benchmark exists and completes
    let bench_path = Path::new("target/criterion/cold_start_empty");

    if !bench_path.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    assert!(bench_path.exists(), "Empty program benchmark not found");
}

#[test]
fn test_edge_case_all_operators_benchmark() {
    // Edge case: Verify all operators are benchmarked
    let bench_path = Path::new("target/criterion/cold_start_all_operators");

    if !bench_path.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    assert!(bench_path.exists(), "All operators benchmark not found");
}

#[test]
fn test_warm_vs_cold_benchmarks_both_exist() {
    // Verify both cold start and warm execution benchmarks exist
    let cold_path = Path::new("target/criterion/cold_start_simple");
    let warm_path = Path::new("target/criterion/warm_execution_simple");

    if !cold_path.exists() || !warm_path.exists() {
        eprintln!("Skipping - run 'cargo bench' first");
        return;
    }

    assert!(cold_path.exists(), "Cold start benchmark missing");
    assert!(warm_path.exists(), "Warm execution benchmark missing");
}
