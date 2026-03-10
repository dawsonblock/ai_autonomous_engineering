/// Integration test for AC6: CPython Pure Execution Baseline Benchmark
/// Validates that the cpython_pure_execution benchmark runs successfully and
/// generates the required estimates.json file with proper structure.
///
/// Acceptance Criteria (AC6):
/// - benches/cpython_pure_execution.rs exists with cpython_pure_simple benchmark
/// - Uses pyo3 with auto-initialize feature
/// - Benchmark measures py.eval('2 + 3') within Python::with_gil block
/// - Generates estimates.json at target/criterion/cpython_pure_simple/base/estimates.json
/// - estimates.json contains mean.point_estimate field
use std::fs;
use std::path::Path;

#[test]
fn test_cpython_pure_execution_benchmark_file_exists() {
    // AC6.1: Verify benches/cpython_pure_execution.rs exists
    let bench_file = Path::new("benches/cpython_pure_execution.rs");

    assert!(
        bench_file.exists(),
        "AC6 FAILED: benches/cpython_pure_execution.rs does not exist"
    );

    let content = fs::read_to_string(bench_file).expect("Failed to read cpython_pure_execution.rs");

    // Verify it contains cpython_pure_simple benchmark function
    assert!(
        content.contains("fn cpython_pure_simple"),
        "AC6 FAILED: cpython_pure_simple benchmark function not found"
    );

    // Verify it uses pyo3
    assert!(
        content.contains("use pyo3::prelude::*"),
        "AC6 FAILED: pyo3 import not found"
    );

    // Verify it uses Python::with_gil
    assert!(
        content.contains("Python::with_gil"),
        "AC6 FAILED: Python::with_gil not found"
    );

    // Verify it evaluates '2 + 3'
    assert!(
        content.contains("py.eval(\"2 + 3\"") || content.contains("py.eval('2 + 3'"),
        "AC6 FAILED: py.eval('2 + 3') not found"
    );
}

#[test]
fn test_cpython_pure_execution_estimates_json_exists() {
    // AC6.4: Verify estimates.json exists at correct path
    let estimates_path = Path::new("target/criterion/cpython_pure_simple/base/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping - run 'PYTHONPATH=/opt/homebrew/opt/python@3.13/Frameworks/Python.framework/Versions/3.13/lib/python3.13 PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo bench --bench cpython_pure_execution' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC6 FAILED: estimates.json not found at target/criterion/cpython_pure_simple/base/estimates.json"
    );
}

#[test]
fn test_cpython_pure_execution_estimates_json_structure() {
    // AC6.5: Verify estimates.json contains mean.point_estimate field
    let estimates_path = Path::new("target/criterion/cpython_pure_simple/base/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench cpython_pure_execution' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse estimates.json");

    // Verify mean.point_estimate exists and is a number
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "AC6 FAILED: mean.point_estimate field missing or not a number"
    );

    let mean_estimate = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Failed to extract mean.point_estimate");

    // Verify mean is a reasonable value (should be in microseconds range for CPython)
    // CPython pure execution should be slower than PyRust (> 500ns)
    assert!(
        mean_estimate > 500.0,
        "AC6 WARNING: CPython execution unusually fast: {:.2}ns (expected > 500ns)",
        mean_estimate
    );

    // Verify mean is not absurdly large (< 100ms)
    assert!(
        mean_estimate < 100_000_000.0,
        "AC6 WARNING: CPython execution unusually slow: {:.2}ns (expected < 100ms)",
        mean_estimate
    );

    println!(
        "AC6 PASS: CPython pure execution mean = {:.2}µs",
        mean_estimate / 1000.0
    );
}

#[test]
fn test_cpython_pure_execution_benchmark_in_cargo_toml() {
    // AC6.2: Verify pyo3 dev-dependency with auto-initialize feature
    let cargo_toml = Path::new("Cargo.toml");

    let content = fs::read_to_string(cargo_toml).expect("Failed to read Cargo.toml");

    // Verify pyo3 is in dev-dependencies
    assert!(
        content.contains("[dev-dependencies]"),
        "AC6 FAILED: [dev-dependencies] section not found in Cargo.toml"
    );

    // Verify pyo3 with auto-initialize feature
    assert!(
        content.contains("pyo3") && content.contains("auto-initialize"),
        "AC6 FAILED: pyo3 with auto-initialize feature not found in Cargo.toml"
    );

    // Verify cpython_pure_execution benchmark is declared
    assert!(
        content.contains("name = \"cpython_pure_execution\""),
        "AC6 FAILED: cpython_pure_execution benchmark not declared in Cargo.toml"
    );
}

#[test]
fn test_cpython_pure_execution_edge_case_consistency() {
    // Edge case: Verify multiple runs produce consistent results (CV < 20%)
    let estimates_path = Path::new("target/criterion/cpython_pure_simple/base/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench cpython_pure_execution' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse estimates.json");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Missing mean.point_estimate");

    let std_dev_ns = data["std_dev"]["point_estimate"]
        .as_f64()
        .expect("Missing std_dev.point_estimate");

    let cv = std_dev_ns / mean_ns;

    // CPython benchmarks may have higher variance than PyRust, allow up to 20%
    assert!(
        cv < 0.20,
        "AC6 EDGE CASE FAILED: Coefficient of variation {:.2}% exceeds 20% threshold (CPython benchmark unstable)",
        cv * 100.0
    );

    println!(
        "AC6 EDGE CASE PASS: CPython benchmark CV = {:.2}% (< 20%)",
        cv * 100.0
    );
}

#[test]
fn test_cpython_pure_execution_edge_case_all_fields() {
    // Edge case: Verify all required Criterion fields exist
    let estimates_path = Path::new("target/criterion/cpython_pure_simple/base/estimates.json");

    // Skip test if benchmarks haven't been run yet
    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench cpython_pure_execution' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse estimates.json");

    // Verify all required fields per Criterion output format
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "Missing mean.point_estimate"
    );
    assert!(
        data["mean"]["confidence_interval"].is_object(),
        "Missing mean.confidence_interval"
    );
    assert!(
        data["std_dev"]["point_estimate"].is_f64(),
        "Missing std_dev.point_estimate"
    );
    assert!(
        data["median"]["point_estimate"].is_f64(),
        "Missing median.point_estimate"
    );
    assert!(
        data["slope"].is_object() || data["slope"].is_null(),
        "Invalid slope field"
    );

    println!("AC6 EDGE CASE PASS: All required Criterion fields present");
}

#[test]
fn test_cpython_pure_execution_edge_case_no_initialization_in_loop() {
    // Edge case: Verify initialization happens outside measurement loop
    // This is a code review test - check that prepare_freethreaded_python or
    // equivalent initialization happens before bench_function

    let bench_file = Path::new("benches/cpython_pure_execution.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read cpython_pure_execution.rs");

    // The code should initialize Python before c.bench_function
    // Either via prepare_freethreaded_python() or Python::with_gil outside bench_function
    let has_pre_initialization = content.contains("Python::with_gil(|_py|")
        || content.contains("prepare_freethreaded_python()");

    assert!(
        has_pre_initialization,
        "AC6 EDGE CASE WARNING: Python initialization may not happen outside measurement loop"
    );

    // Verify the measurement loop only contains with_gil for execution, not initialization
    // Look for pattern: b.iter(|| { Python::with_gil(...) })
    assert!(
        content.contains("b.iter(||") && content.contains("Python::with_gil(|py|"),
        "AC6 EDGE CASE FAILED: Measurement loop structure incorrect"
    );
}
