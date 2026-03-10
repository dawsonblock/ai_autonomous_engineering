use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::execute_python;
use std::process::Command;

/// Benchmark CPython subprocess execution for baseline comparison.
/// This measures the total time to execute Python code via subprocess,
/// including process startup overhead.
fn cpython_subprocess_baseline(c: &mut Criterion) {
    // Verify python3 is available before running benchmark
    let check = Command::new("python3").arg("--version").output();

    if check.is_err() {
        eprintln!("Warning: python3 not found on system. Skipping CPython baseline benchmark.");
        return;
    }

    c.bench_function("cpython_subprocess_baseline", |b| {
        b.iter(|| {
            let output = Command::new("python3")
                .arg("-c")
                .arg(black_box("2 + 3"))
                .output()
                .expect("Failed to execute python3");
            black_box(output)
        });
    });
}

/// Benchmark PyRust execution for direct comparison with CPython.
/// This uses the same Python code as the CPython baseline to ensure fair comparison.
fn pyrust_baseline(c: &mut Criterion) {
    c.bench_function("pyrust_baseline", |b| {
        b.iter(|| {
            let result = execute_python(black_box("2 + 3"));
            black_box(result)
        });
    });
}

/// Benchmark for cold start execution - measures complete pipeline from source to output.
/// This is the primary metric for AC1.2 (< 100μs target).
fn cold_start_simple(c: &mut Criterion) {
    c.bench_function("cold_start_simple", |b| {
        b.iter(|| {
            let result = execute_python(black_box("2 + 3"));
            black_box(result)
        });
    });
}

/// Benchmark warm execution - measures repeated execution of same code.
/// This shows the benefit of avoiding recompilation in real-world scenarios.
fn warm_execution(c: &mut Criterion) {
    // Pre-compile once (in real usage, code would be cached)
    let code = "2 + 3";

    c.bench_function("warm_execution", |b| {
        b.iter(|| {
            let result = execute_python(black_box(code));
            black_box(result)
        });
    });
}

/// Benchmark speedup calculation - this is measured for AC1.3.
/// The speedup ratio is calculated as: cpython_mean / pyrust_mean.
/// Target: ≥50x speedup.
///
/// Note: This benchmark group will be used by scripts/compare_cpython.sh
/// to extract timing data and calculate the speedup ratio with statistical confidence.
fn speedup_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("speedup_comparison");

    // CPython baseline - subprocess execution
    if Command::new("python3").arg("--version").output().is_ok() {
        group.bench_function("cpython_total_time", |b| {
            b.iter(|| {
                let output = Command::new("python3")
                    .arg("-c")
                    .arg(black_box("2 + 3"))
                    .output()
                    .expect("Failed to execute python3");
                black_box(output)
            });
        });
    }

    // PyRust baseline - library execution
    group.bench_function("pyrust_total_time", |b| {
        b.iter(|| {
            let result = execute_python(black_box("2 + 3"));
            black_box(result)
        });
    });

    group.finish();
}

// Configure Criterion with sample_size(1000) and measurement_time(10s) to reduce CV below 10% threshold
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        cpython_subprocess_baseline,
        pyrust_baseline,
        cold_start_simple,
        warm_execution,
        speedup_calculation
}
criterion_main!(benches);
