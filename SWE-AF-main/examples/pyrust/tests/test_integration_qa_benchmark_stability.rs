//! Integration QA: Benchmark Stability Tests
//!
//! Tests benchmark configuration changes (issue 08) don't break existing functionality
//! and that all modified benchmark files work with the execution pipeline
//!
//! PRIORITY 1: Test that benchmark configurations don't affect library behavior
//! PRIORITY 2: Test execution consistency for benchmark workloads
//! PRIORITY 3: Test all benchmark file scenarios work correctly

use pyrust::execute_python;

/// Test simple arithmetic workload (used in benchmarks)
#[test]
fn test_benchmark_workload_simple_arithmetic() {
    let code = "2 + 2";
    let result = execute_python(code).expect("Simple arithmetic failed");
    assert_eq!(result, "4");
}

/// Test lexer benchmark workload
#[test]
fn test_benchmark_workload_lexer_scenarios() {
    // Simple expression
    let result = execute_python("42").expect("Integer literal failed");
    assert_eq!(result, "42");

    // Operators
    let result = execute_python("1 + 2 * 3").expect("Operators failed");
    assert_eq!(result, "7");

    // Variables
    let result = execute_python("x = 10\ny = 20\nx + y").expect("Variables failed");
    assert_eq!(result, "30");
}

/// Test parser benchmark workload
#[test]
fn test_benchmark_workload_parser_scenarios() {
    // Nested parentheses
    let result = execute_python("((1 + 2) * (3 + 4))").expect("Nested parens failed");
    assert_eq!(result, "21");

    // Multiple statements
    let code = "x = 1\ny = 2\nz = 3\nx + y + z";
    let result = execute_python(code).expect("Multiple statements failed");
    assert_eq!(result, "6");
}

/// Test compiler benchmark workload
#[test]
fn test_benchmark_workload_compiler_scenarios() {
    // Variable assignment and usage
    let code = "a = 10\nb = 20\nc = 30\na + b + c";
    let result = execute_python(code).expect("Compiler workload failed");
    assert_eq!(result, "60");

    // Complex expressions
    let code = "x = 5\ny = x * 2\nz = y + 3\nz";
    let result = execute_python(code).expect("Complex expression failed");
    assert_eq!(result, "13");
}

/// Test VM execution benchmark workload
#[test]
fn test_benchmark_workload_vm_execution() {
    // Arithmetic operations
    let result = execute_python("10 + 5 * 2").expect("VM arithmetic failed");
    assert_eq!(result, "20");

    // Division
    let result = execute_python("100 / 10").expect("VM division failed");
    assert_eq!(result, "10");

    // Modulo
    let result = execute_python("17 % 5").expect("VM modulo failed");
    assert_eq!(result, "2");
}

/// Test function call overhead benchmark workload
#[test]
fn test_benchmark_workload_function_calls() {
    let code = r#"
def add(a, b):
    return a + b

add(10, 20)
"#;
    let result = execute_python(code).expect("Function call failed");
    assert_eq!(result, "30");

    // Multiple function calls
    let code = r#"
def mul(x, y):
    return x * y

a = mul(2, 3)
b = mul(4, 5)
a + b
"#;
    let result = execute_python(code).expect("Multiple calls failed");
    assert_eq!(result, "26");
}

/// Test startup benchmark scenario (simple execution)
#[test]
fn test_benchmark_workload_startup() {
    // Very simple program for startup measurement
    let result = execute_python("42").expect("Startup workload failed");
    assert_eq!(result, "42");

    let result = execute_python("1 + 1").expect("Minimal arithmetic failed");
    assert_eq!(result, "2");
}

/// Test cache performance benchmark workload
#[test]
fn test_benchmark_workload_cache_performance() {
    use pyrust::{clear_thread_local_cache, execute_python_cached};

    clear_thread_local_cache();

    let code = "2 + 3";

    // First execution (cache miss)
    let result1 = execute_python_cached(code).expect("Cache miss failed");
    assert_eq!(result1, "5");

    // Second execution (cache hit)
    let result2 = execute_python_cached(code).expect("Cache hit failed");
    assert_eq!(result2, "5");

    // Results should be identical
    assert_eq!(result1, result2);
}

/// Test profiling overhead benchmark workload
#[test]
fn test_benchmark_workload_profiling_overhead() {
    use pyrust::profiling::execute_python_profiled;

    let code = "10 * 10";

    // Execute with profiling
    let (output, profile) = execute_python_profiled(code).expect("Profiled execution failed");
    assert_eq!(output, "100");

    // Verify profiling captured data
    assert!(profile.total_ns > 0, "Total time should be recorded");
    assert!(profile.lex_ns > 0, "Lex time should be recorded");
    assert!(profile.parse_ns > 0, "Parse time should be recorded");
    assert!(profile.compile_ns > 0, "Compile time should be recorded");
    assert!(profile.vm_execute_ns > 0, "VM time should be recorded");
}

/// Test consistency across multiple executions (benchmark stability requirement)
#[test]
fn test_benchmark_workload_execution_consistency() {
    let code = "7 * 8";

    // Execute 10 times and verify consistent results
    for _ in 0..10 {
        let result = execute_python(code).expect("Execution failed");
        assert_eq!(
            result, "56",
            "Results should be consistent across executions"
        );
    }
}

/// Test all operators work (benchmark coverage)
#[test]
fn test_benchmark_workload_all_operators() {
    assert_eq!(execute_python("10 + 5").unwrap(), "15");
    assert_eq!(execute_python("10 - 5").unwrap(), "5");
    assert_eq!(execute_python("10 * 5").unwrap(), "50");
    assert_eq!(execute_python("10 / 5").unwrap(), "2");
    assert_eq!(execute_python("10 // 3").unwrap(), "3");
    assert_eq!(execute_python("10 % 3").unwrap(), "1");
}

/// Test print statement workload (used in benchmarks)
#[test]
fn test_benchmark_workload_print_statements() {
    let result = execute_python("print(42)").expect("Print failed");
    assert_eq!(result, "42\n");

    let result = execute_python("print(1)\nprint(2)").expect("Multiple prints failed");
    assert_eq!(result, "1\n2\n");
}

/// Test mixed workload (statements + expressions)
#[test]
fn test_benchmark_workload_mixed_statements() {
    let code = r#"
x = 10
y = 20
print(x)
print(y)
x + y
"#;
    let result = execute_python(code).expect("Mixed workload failed");
    assert_eq!(result, "10\n20\n30");
}

/// Test that error handling works correctly under benchmark conditions
#[test]
fn test_benchmark_workload_error_handling() {
    // Division by zero
    let result = execute_python("10 / 0");
    assert!(result.is_err(), "Should error on division by zero");

    // Undefined variable
    let result = execute_python("undefined_var");
    assert!(result.is_err(), "Should error on undefined variable");

    // Syntax error
    let result = execute_python("1 +");
    assert!(result.is_err(), "Should error on syntax error");
}

/// Test that benchmark configurations don't break edge cases
#[test]
fn test_benchmark_workload_edge_cases() {
    // Empty program
    let result = execute_python("").expect("Empty program failed");
    assert_eq!(result, "");

    // Zero
    let result = execute_python("0").expect("Zero failed");
    assert_eq!(result, "0");

    // Large number
    let result = execute_python("1000000").expect("Large number failed");
    assert_eq!(result, "1000000");
}

/// Test concurrent-like execution pattern (stress test)
#[test]
fn test_benchmark_workload_stress_repeated_execution() {
    let code = "123 + 456";

    // Execute 100 times rapidly
    for _ in 0..100 {
        let result = execute_python(code).expect("Stress execution failed");
        assert_eq!(result, "579");
    }
}

/// Test function parameter scenarios for benchmark workloads
#[test]
fn test_benchmark_workload_function_parameters() {
    let code = r#"
def compute(a, b, c):
    return a * b + c

compute(5, 6, 10)
"#;
    let result = execute_python(code).expect("Function params failed");
    assert_eq!(result, "40");
}

/// Test negative numbers in benchmark workloads
#[test]
fn test_benchmark_workload_negative_numbers() {
    let result = execute_python("-42").expect("Negative literal failed");
    assert_eq!(result, "-42");

    let result = execute_python("10 - 30").expect("Negative result failed");
    assert_eq!(result, "-20");
}

/// Test that benchmark Criterion configuration changes don't affect results
#[test]
fn test_benchmark_workload_criterion_compatibility() {
    // These workloads should work regardless of Criterion configuration
    let workloads = vec![
        ("2+2", "4"),
        ("10*10", "100"),
        ("x=5\nx", "5"),
        ("print(99)", "99\n"),
    ];

    for (code, expected) in workloads {
        let result = execute_python(code).expect(&format!("Workload failed: {}", code));
        assert_eq!(result, expected, "Workload mismatch: {}", code);
    }
}
