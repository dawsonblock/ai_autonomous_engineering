use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyo3::prelude::*;

/// Benchmark CPython pure execution using pyo3 Python C API.
/// This measures execution time excluding interpreter startup overhead.
/// Uses Python::with_gil for repeated eval() calls to isolate pure execution time.
/// Critical for AC6 (50x speedup validation against CPython baseline).
fn cpython_pure_simple(c: &mut Criterion) {
    // Initialize Python once outside measurement loop (equivalent to prepare_freethreaded_python)
    // The auto-initialize feature handles initialization on first with_gil call
    // This ensures we only measure execution time, not startup overhead
    Python::with_gil(|_py| {
        // Python interpreter is now initialized
    });

    c.bench_function("cpython_pure_simple", |b| {
        b.iter(|| {
            // Measure py.eval('2 + 3') within Python::with_gil block
            // This isolates pure execution time without subprocess or interpreter startup
            Python::with_gil(|py| {
                let result: i64 = py.eval("2 + 3", None, None).unwrap().extract().unwrap();
                black_box(result)
            })
        });
    });
}

// Configure Criterion with sample_size(1000) and measurement_time(10s) to reduce CV below 10% threshold
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets = cpython_pure_simple
}
criterion_main!(benches);
