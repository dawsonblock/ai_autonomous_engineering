use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::{lexer, parser};

/// Benchmark parser-only performance: simple arithmetic (2 + 3)
/// Pre-tokenizes input to isolate parser from lexer performance
fn parser_simple(c: &mut Criterion) {
    // Pre-tokenize outside the benchmark loop to isolate parser performance
    let tokens = lexer::lex("2 + 3").unwrap();

    c.bench_function("parser_simple", |b| {
        b.iter(|| {
            // Clone tokens for each iteration
            let tokens_clone = tokens.clone();
            let result = parser::parse(black_box(tokens_clone));
            black_box(result)
        });
    });
}

/// Benchmark parser-only performance: complex arithmetic expression
/// Tests: (10 + 20) * 3 / 2 - 8 % 4
fn parser_complex(c: &mut Criterion) {
    // Pre-tokenize outside the benchmark loop to isolate parser performance
    let tokens = lexer::lex("(10 + 20) * 3 / 2 - 8 % 4").unwrap();

    c.bench_function("parser_complex", |b| {
        b.iter(|| {
            // Clone tokens for each iteration
            let tokens_clone = tokens.clone();
            let result = parser::parse(black_box(tokens_clone));
            black_box(result)
        });
    });
}

/// Benchmark parser-only performance: with variables and assignments
/// Tests: x = 42
fn parser_variables(c: &mut Criterion) {
    // Pre-tokenize outside the benchmark loop to isolate parser performance
    let tokens = lexer::lex("x = 42").unwrap();

    c.bench_function("parser_variables", |b| {
        b.iter(|| {
            // Clone tokens for each iteration
            let tokens_clone = tokens.clone();
            let result = parser::parse(black_box(tokens_clone));
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
        parser_simple,
        parser_complex,
        parser_variables
}
criterion_main!(benches);
