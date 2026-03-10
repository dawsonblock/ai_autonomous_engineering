use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::{compiler, lexer, parser, vm::VM};

/// Benchmark warm execution: simple arithmetic (2 + 3)
/// This measures just the VM execution time with pre-compiled bytecode
fn warm_execution_simple(c: &mut Criterion) {
    // Pre-compile the bytecode
    let tokens = lexer::lex("2 + 3").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_simple", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: complex arithmetic
/// Tests: (10 + 20) * 3 / 2
fn warm_execution_complex(c: &mut Criterion) {
    let tokens = lexer::lex("(10 + 20) * 3 / 2").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_complex", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: with variables
/// Tests: x = 10; y = 20; x + y
fn warm_execution_with_variables(c: &mut Criterion) {
    let tokens = lexer::lex("x = 10\ny = 20\nx + y").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_with_variables", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: with print
/// Tests: print(42)
fn warm_execution_with_print(c: &mut Criterion) {
    let tokens = lexer::lex("print(42)").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_with_print", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: empty program
fn warm_execution_empty(c: &mut Criterion) {
    let tokens = lexer::lex("").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_empty", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: all operators
/// Tests: 10 + 5 * 2 - 8 / 4 % 3
fn warm_execution_all_operators(c: &mut Criterion) {
    let tokens = lexer::lex("10 + 5 * 2 - 8 / 4 % 3").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_all_operators", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: deeply nested expression
/// Tests: ((1 + 2) * (3 + 4)) / 7
fn warm_execution_nested(c: &mut Criterion) {
    let tokens = lexer::lex("((1 + 2) * (3 + 4)) / 7").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_nested", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: floor division
/// Tests: 10 // 3
fn warm_execution_floor_division(c: &mut Criterion) {
    let tokens = lexer::lex("10 // 3").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_floor_division", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: modulo
/// Tests: 10 % 3
fn warm_execution_modulo(c: &mut Criterion) {
    let tokens = lexer::lex("10 % 3").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_modulo", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            let result = vm.execute(black_box(&bytecode));
            black_box(result)
        });
    });
}

/// Benchmark warm execution: complex program
/// Tests: x = 10\nprint(x)\ny = 20\nprint(y)\nx + y
fn warm_execution_complex_program(c: &mut Criterion) {
    let tokens = lexer::lex("x = 10\nprint(x)\ny = 20\nprint(y)\nx + y").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    c.bench_function("warm_execution_complex_program", |b| {
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
        warm_execution_simple,
        warm_execution_complex,
        warm_execution_with_variables,
        warm_execution_empty,
        warm_execution_all_operators,
        warm_execution_floor_division,
        warm_execution_modulo,
        warm_execution_complex_program
}

// Separate group for warm_execution_with_print with higher sample_size(2000) and measurement_time(20s)
// to achieve CV < 10% (was 48.13% with default settings)
criterion_group! {
    name = benches_print;
    config = Criterion::default()
        .sample_size(2000)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        warm_execution_with_print
}

// Separate group for warm_execution_nested with higher sample_size(2000) and measurement_time(20s)
// to achieve CV < 10% (has high variance due to nested expression evaluation)
criterion_group! {
    name = benches_nested;
    config = Criterion::default()
        .sample_size(2000)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        warm_execution_nested
}

criterion_main!(benches, benches_print, benches_nested);
