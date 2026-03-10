use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyrust::execute_python;

/// Benchmark direct arithmetic: 10 + 20
/// This is the baseline for measuring function call overhead
fn direct_arithmetic(c: &mut Criterion) {
    c.bench_function("direct_arithmetic", |b| {
        b.iter(|| {
            let result = execute_python(black_box("10 + 20"));
            black_box(result)
        });
    });
}

/// Benchmark function call with arithmetic: add(10, 20)
/// This measures the overhead of calling a function vs direct arithmetic
fn function_call_arithmetic(c: &mut Criterion) {
    c.bench_function("function_call_arithmetic", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def add(a, b):\n    return a + b\nadd(10, 20)"));
            black_box(result)
        });
    });
}

/// Benchmark function call overhead
/// Measures the difference between function call and direct arithmetic
/// Target: < 5μs overhead (AC2.8)
fn function_call_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_call_overhead");

    // Baseline: direct arithmetic
    group.bench_function("baseline_arithmetic", |b| {
        b.iter(|| {
            let result = execute_python(black_box("10 + 20"));
            black_box(result)
        });
    });

    // Function call: same arithmetic through function
    group.bench_function("with_function_call", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def add(a, b):\n    return a + b\nadd(10, 20)"));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark zero-parameter function call
fn zero_param_function_call(c: &mut Criterion) {
    c.bench_function("zero_param_function_call", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def foo():\n    return 42\nfoo()"));
            black_box(result)
        });
    });
}

/// Benchmark single-parameter function call
fn single_param_function_call(c: &mut Criterion) {
    c.bench_function("single_param_function_call", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def double(x):\n    return x * 2\ndouble(21)"));
            black_box(result)
        });
    });
}

/// Benchmark three-parameter function call
fn three_param_function_call(c: &mut Criterion) {
    c.bench_function("three_param_function_call", |b| {
        b.iter(|| {
            let result = execute_python(black_box(
                "def add_three(a, b, c):\n    return a + b + c\nadd_three(10, 20, 30)",
            ));
            black_box(result)
        });
    });
}

/// Benchmark nested function calls
fn nested_function_calls(c: &mut Criterion) {
    c.bench_function("nested_function_calls", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def double(x):\n    return x * 2\ndef quad(x):\n    return double(double(x))\nquad(5)"));
            black_box(result)
        });
    });
}

/// Benchmark multiple sequential function calls
fn multiple_sequential_calls(c: &mut Criterion) {
    c.bench_function("multiple_sequential_calls", |b| {
        b.iter(|| {
            let result = execute_python(black_box(
                "def add(a, b):\n    return a + b\nadd(1, 2)\nadd(3, 4)\nadd(5, 6)",
            ));
            black_box(result)
        });
    });
}

/// Benchmark function with local variables
fn function_with_local_vars(c: &mut Criterion) {
    c.bench_function("function_with_local_vars", |b| {
        b.iter(|| {
            let result = execute_python(black_box(
                "def calc():\n    x = 10\n    y = 20\n    z = x + y\n    return z\ncalc()",
            ));
            black_box(result)
        });
    });
}

/// Benchmark function with complex computation
fn function_with_complex_computation(c: &mut Criterion) {
    c.bench_function("function_with_complex_computation", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def complex(a, b, c):\n    x = a + b * c\n    y = x / b\n    return y\ncomplex(10, 5, 2)"));
            black_box(result)
        });
    });
}

/// Benchmark function call vs direct for simple return
fn simple_return_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_return_overhead");

    // Baseline: direct literal
    group.bench_function("direct_literal", |b| {
        b.iter(|| {
            let result = execute_python(black_box("42"));
            black_box(result)
        });
    });

    // Function returning literal
    group.bench_function("function_return_literal", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def get_value():\n    return 42\nget_value()"));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark function definition overhead
fn function_definition_only(c: &mut Criterion) {
    c.bench_function("function_definition_only", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def foo():\n    return 42"));
            black_box(result)
        });
    });
}

/// Benchmark function with print statement
fn function_with_print(c: &mut Criterion) {
    c.bench_function("function_with_print", |b| {
        b.iter(|| {
            let result = execute_python(black_box(
                "def print_val():\n    print(42)\n    return 100\nprint_val()",
            ));
            black_box(result)
        });
    });
}

/// Benchmark function call in assignment
fn function_call_in_assignment(c: &mut Criterion) {
    c.bench_function("function_call_in_assignment", |b| {
        b.iter(|| {
            let result =
                execute_python(black_box("def get_val():\n    return 42\nx = get_val()\nx"));
            black_box(result)
        });
    });
}

/// Benchmark function with all arithmetic operators
fn function_with_all_operators(c: &mut Criterion) {
    c.bench_function("function_with_all_operators", |b| {
        b.iter(|| {
            let result = execute_python(black_box("def all_ops(a, b):\n    x = a + b\n    y = a - b\n    z = a * b\n    w = a / b\n    return w\nall_ops(20, 4)"));
            black_box(result)
        });
    });
}

/// Benchmark recursive function call (simple countdown)
fn recursive_function_call(c: &mut Criterion) {
    c.bench_function("recursive_function_call", |b| {
        b.iter(|| {
            // Note: This assumes conditionals are implemented
            // If not, this test may fail - that's expected
            let result = execute_python(black_box("def countdown(n):\n    if n <= 0:\n        return 0\n    return n + countdown(n - 1)\ncountdown(5)"));
            // Don't fail if conditionals aren't implemented
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
        direct_arithmetic,
        function_call_arithmetic,
        function_call_overhead,
        zero_param_function_call,
        single_param_function_call,
        three_param_function_call,
        nested_function_calls,
        multiple_sequential_calls,
        function_with_local_vars,
        function_with_complex_computation,
        simple_return_overhead,
        function_definition_only,
        function_call_in_assignment,
        recursive_function_call
}

// Separate group for function_with_all_operators with measurement_time(15s)
// to achieve CV < 10% (was 10.99% with 10s measurement time)
criterion_group! {
    name = benches_all_ops;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(15))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        function_with_all_operators
}

// Separate group for function_with_print with higher sample_size(2000) and measurement_time(20s)
// to achieve CV < 10% (has high variance due to print statement overhead)
criterion_group! {
    name = benches_print;
    config = Criterion::default()
        .sample_size(2000)
        .measurement_time(std::time::Duration::from_secs(20))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        function_with_print
}

criterion_main!(benches, benches_all_ops, benches_print);
