use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::{compiler, lexer, parser};

/// Benchmark compiler: simple arithmetic (2 + 3)
/// Pre-parses AST outside benchmark loop to isolate compiler performance
fn compiler_simple(c: &mut Criterion) {
    // Pre-parse the AST outside the benchmark loop
    let tokens = lexer::lex("2 + 3").unwrap();
    let ast = parser::parse(tokens).unwrap();

    c.bench_function("compiler_simple", |b| {
        b.iter(|| {
            let bytecode = compiler::compile(black_box(&ast));
            black_box(bytecode)
        });
    });
}

/// Benchmark compiler: complex arithmetic expression
/// Tests: (10 + 20) * 3 / 2
/// Pre-parses AST outside benchmark loop to isolate compiler performance
fn compiler_complex(c: &mut Criterion) {
    // Pre-parse the AST: (10 + 20) * 3 / 2
    let tokens = lexer::lex("(10 + 20) * 3 / 2").unwrap();
    let ast = parser::parse(tokens).unwrap();

    c.bench_function("compiler_complex", |b| {
        b.iter(|| {
            let bytecode = compiler::compile(black_box(&ast));
            black_box(bytecode)
        });
    });
}

/// Benchmark compiler: with variables
/// Tests: x = 10; y = 20; x + y
/// Pre-parses AST outside benchmark loop to isolate compiler performance
fn compiler_variables(c: &mut Criterion) {
    // Pre-parse the AST: x = 10; y = 20; x + y
    let tokens = lexer::lex("x = 10\ny = 20\nx + y").unwrap();
    let ast = parser::parse(tokens).unwrap();

    c.bench_function("compiler_variables", |b| {
        b.iter(|| {
            let bytecode = compiler::compile(black_box(&ast));
            black_box(bytecode)
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
        compiler_simple,
        compiler_complex
}

// Separate group for compiler_variables with higher sample_size(2000) and measurement_time(15s)
// to achieve CV < 5% (was 5.74% with standard settings)
criterion_group! {
    name = benches_variables;
    config = Criterion::default()
        .sample_size(2000)
        .measurement_time(std::time::Duration::from_secs(15))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        compiler_variables
}

criterion_main!(benches, benches_variables);
