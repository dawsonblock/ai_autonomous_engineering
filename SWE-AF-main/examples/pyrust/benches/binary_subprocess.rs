use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the release binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("pyrust");
    path
}

/// Benchmark: Binary subprocess execution for simple arithmetic (AC6.1 - target ≤380μs)
/// This measures end-to-end binary execution including process spawn, execution, and output
fn bench_binary_subprocess_simple(c: &mut Criterion) {
    let binary_path = get_binary_path();

    // Verify binary exists
    assert!(
        binary_path.exists(),
        "Binary not found at {:?}. Run 'cargo build --release' first.",
        binary_path
    );

    c.bench_function("binary_subprocess_simple_arithmetic", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("2+3"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "5", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution for complex expression
fn bench_binary_subprocess_complex(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_complex_expression", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("(10 + 20) * 3 / 2"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "45", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution with variables
fn bench_binary_subprocess_variables(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_with_variables", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("x = 10\ny = 20\nx + y"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "30", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution with print statement
fn bench_binary_subprocess_print(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_with_print", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("print(42)"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "42", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution with multiple operations
fn bench_binary_subprocess_multiple_ops(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_multiple_operations", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("10 + 5 * 2 - 8 / 4 % 3"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "18", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution with nested expressions
fn bench_binary_subprocess_nested(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_nested_expression", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("((1 + 2) * (3 + 4)) / 7"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "3", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution with floor division
fn bench_binary_subprocess_floor_division(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_floor_division", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("10 // 3"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "3", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution with modulo
fn bench_binary_subprocess_modulo(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_modulo", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box("10 % 3"))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(stdout.trim(), "1", "Unexpected output: {}", stdout);
        });
    });
}

/// Benchmark: Binary subprocess execution parameterized by code complexity
fn bench_binary_subprocess_by_complexity(c: &mut Criterion) {
    let binary_path = get_binary_path();
    let mut group = c.benchmark_group("binary_subprocess_by_complexity");

    let test_cases = vec![
        ("minimal", "42", "42"),
        ("simple_arithmetic", "2+3", "5"),
        ("medium_expression", "(10 + 20) * 3", "90"),
        ("complex_program", "x = 10\ny = 20\nz = x + y\nz * 2", "60"),
    ];

    for (name, code, expected) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, &code| {
            b.iter(|| {
                let output = Command::new(&binary_path)
                    .arg("-c")
                    .arg(black_box(code))
                    .output()
                    .expect("Failed to execute binary");

                assert!(output.status.success(), "Binary execution failed");
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert_eq!(stdout.trim(), expected, "Unexpected output: {}", stdout);
            });
        });
    }

    group.finish();
}

/// Benchmark: Binary subprocess execution measuring startup overhead
/// This uses an empty program to isolate startup costs
fn bench_binary_subprocess_startup_overhead(c: &mut Criterion) {
    let binary_path = get_binary_path();

    c.bench_function("binary_subprocess_startup_overhead", |b| {
        b.iter(|| {
            let output = Command::new(&binary_path)
                .arg("-c")
                .arg(black_box(""))
                .output()
                .expect("Failed to execute binary");

            assert!(output.status.success(), "Binary execution failed");
        });
    });
}

// Configure Criterion with sample_size(1000) and measurement_time(15s) to reduce CV below 10% threshold
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(15))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        bench_binary_subprocess_simple,
        bench_binary_subprocess_complex,
        bench_binary_subprocess_variables,
        bench_binary_subprocess_print,
        bench_binary_subprocess_multiple_ops,
        bench_binary_subprocess_nested,
        bench_binary_subprocess_floor_division,
        bench_binary_subprocess_modulo,
        bench_binary_subprocess_by_complexity,
        bench_binary_subprocess_startup_overhead
}

criterion_main!(benches);
