use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pyrust::{execute_python, profiling::execute_python_profiled};

/// Benchmark profiling overhead by comparing execute_python vs execute_python_profiled
/// AC5.4: Profiling overhead ≤1% (note: architecture revision updated to ≤20%)
fn benchmark_profiling_overhead(c: &mut Criterion) {
    let test_cases = vec![
        ("simple_expr", "2+3"),
        ("variable_assign", "x = 10\ny = 20\nx + y"),
        ("print_stmt", "print(42)"),
        ("complex_expr", "((1 + 2) * (3 + 4)) / 7"),
        ("arithmetic", "10 + 5 * 2 - 8 / 4 % 3"),
    ];

    for (name, code) in test_cases {
        let mut group = c.benchmark_group("profiling_overhead");

        // Benchmark normal execution
        group.bench_with_input(BenchmarkId::new("normal", name), &code, |b, &code| {
            b.iter(|| black_box(execute_python(code).unwrap()));
        });

        // Benchmark profiled execution
        group.bench_with_input(BenchmarkId::new("profiled", name), &code, |b, &code| {
            b.iter(|| black_box(execute_python_profiled(code).unwrap()));
        });

        group.finish();
    }
}

/// Measure absolute overhead in nanoseconds
fn benchmark_profiling_overhead_absolute(c: &mut Criterion) {
    let code = "2+3";

    c.bench_function("overhead_normal_execution", |b| {
        b.iter(|| black_box(execute_python(code).unwrap()));
    });

    c.bench_function("overhead_profiled_execution", |b| {
        b.iter(|| black_box(execute_python_profiled(code).unwrap()));
    });
}

/// Verify profiling correctness under benchmark conditions
fn benchmark_profiling_accuracy(c: &mut Criterion) {
    c.bench_function("profiling_sum_validation", |b| {
        b.iter(|| {
            let (_, profile) = execute_python_profiled("2+3").unwrap();
            black_box(profile.validate_timing_sum())
        });
    });
}

// Configure Criterion with sample_size(1000) and measurement_time(10s) to reduce CV below 10% threshold
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets = benchmark_profiling_overhead,
              benchmark_profiling_overhead_absolute,
              benchmark_profiling_accuracy
);
criterion_main!(benches);
