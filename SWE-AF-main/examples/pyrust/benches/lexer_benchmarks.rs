use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::lexer;

/// Benchmark lexer: simple arithmetic expression (2 + 3)
/// Measures tokenization time for basic expression
fn lexer_simple(c: &mut Criterion) {
    c.bench_function("lexer_simple", |b| {
        b.iter(|| {
            let result = lexer::lex(black_box("2 + 3"));
            black_box(result)
        });
    });
}

/// Benchmark lexer: complex arithmetic expression
/// Tests: (10 + 20) * 3 / 2
/// Measures tokenization time for nested expression with multiple operators
fn lexer_complex(c: &mut Criterion) {
    c.bench_function("lexer_complex", |b| {
        b.iter(|| {
            let result = lexer::lex(black_box("(10 + 20) * 3 / 2"));
            black_box(result)
        });
    });
}

/// Benchmark lexer: expression with variables
/// Tests: x = 10; y = 20; x + y
/// Measures tokenization time for variable assignments and arithmetic
fn lexer_variables(c: &mut Criterion) {
    c.bench_function("lexer_variables", |b| {
        b.iter(|| {
            let result = lexer::lex(black_box("x = 10\ny = 20\nx + y"));
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
        lexer_simple,
        lexer_complex,
        lexer_variables
}
criterion_main!(benches);
