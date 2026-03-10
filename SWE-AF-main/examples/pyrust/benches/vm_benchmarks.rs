use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::{compiler, lexer, parser, vm::VM};

/// Benchmark VM execution only: simple arithmetic (2 + 3)
/// Pre-compiles bytecode outside benchmark loop to isolate VM performance
fn vm_simple(c: &mut Criterion) {
    // Pre-compile the bytecode outside the benchmark loop
    let tokens = lexer::lex("2 + 3").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("vm_simple", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark VM execution only: complex arithmetic
/// Tests: (10 + 20) * 3 / 2
/// Pre-compiles bytecode to measure only VM execution time
fn vm_complex(c: &mut Criterion) {
    // Pre-compile the bytecode outside the benchmark loop
    let tokens = lexer::lex("(10 + 20) * 3 / 2").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("vm_complex", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark VM execution only: with variables
/// Tests: x = 10; y = 20; x + y
/// Pre-compiles bytecode to measure only VM execution time
fn vm_variables(c: &mut Criterion) {
    // Pre-compile the bytecode outside the benchmark loop
    let tokens = lexer::lex("x = 10\ny = 20\nx + y").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("vm_variables", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
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
        vm_simple,
        vm_complex,
        vm_variables
}
criterion_main!(benches);
