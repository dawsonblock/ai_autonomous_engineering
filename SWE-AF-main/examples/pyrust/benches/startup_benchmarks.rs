use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::execute_python;

/// Benchmark cold start: simple arithmetic (2 + 3)
/// This is the critical benchmark for AC1.2 validation (< 100μs target)
fn cold_start_simple(c: &mut Criterion) {
    c.bench_function("cold_start_simple", |b| {
        b.iter(|| {
            let result = execute_python(black_box("2 + 3"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: complex arithmetic expression
/// Tests: (10 + 20) * 3 / 2
fn cold_start_complex(c: &mut Criterion) {
    c.bench_function("cold_start_complex", |b| {
        b.iter(|| {
            let result = execute_python(black_box("(10 + 20) * 3 / 2"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: with variables
/// Tests: x = 10; y = 20; x + y
fn with_variables(c: &mut Criterion) {
    c.bench_function("with_variables", |b| {
        b.iter(|| {
            let result = execute_python(black_box("x = 10\ny = 20\nx + y"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: with print statement
/// Tests: print(42)
fn with_print(c: &mut Criterion) {
    c.bench_function("with_print", |b| {
        b.iter(|| {
            let result = execute_python(black_box("print(42)"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: empty program
/// Tests edge case of minimal execution path
fn cold_start_empty(c: &mut Criterion) {
    c.bench_function("cold_start_empty", |b| {
        b.iter(|| {
            let result = execute_python(black_box(""));
            black_box(result)
        });
    });
}

/// Benchmark cold start: complex program with multiple operations
/// Tests: x = 10\nprint(x)\ny = 20\nprint(y)\nx + y
fn cold_start_complex_program(c: &mut Criterion) {
    c.bench_function("cold_start_complex_program", |b| {
        b.iter(|| {
            let result = execute_python(black_box("x = 10\nprint(x)\ny = 20\nprint(y)\nx + y"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: all arithmetic operators
/// Tests: 10 + 5 * 2 - 8 / 4 % 3
fn cold_start_all_operators(c: &mut Criterion) {
    c.bench_function("cold_start_all_operators", |b| {
        b.iter(|| {
            let result = execute_python(black_box("10 + 5 * 2 - 8 / 4 % 3"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: deeply nested expression
/// Tests: ((1 + 2) * (3 + 4)) / 7
fn cold_start_nested_expression(c: &mut Criterion) {
    c.bench_function("cold_start_nested_expression", |b| {
        b.iter(|| {
            let result = execute_python(black_box("((1 + 2) * (3 + 4)) / 7"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: floor division
/// Tests: 10 // 3
fn cold_start_floor_division(c: &mut Criterion) {
    c.bench_function("cold_start_floor_division", |b| {
        b.iter(|| {
            let result = execute_python(black_box("10 // 3"));
            black_box(result)
        });
    });
}

/// Benchmark cold start: modulo operation
/// Tests: 10 % 3
fn cold_start_modulo(c: &mut Criterion) {
    c.bench_function("cold_start_modulo", |b| {
        b.iter(|| {
            let result = execute_python(black_box("10 % 3"));
            black_box(result)
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
    targets =
        cold_start_simple,
        cold_start_complex,
        with_variables,
        with_print,
        cold_start_empty,
        cold_start_complex_program,
        cold_start_all_operators,
        cold_start_nested_expression,
        cold_start_floor_division,
        cold_start_modulo
}
criterion_main!(benches);
