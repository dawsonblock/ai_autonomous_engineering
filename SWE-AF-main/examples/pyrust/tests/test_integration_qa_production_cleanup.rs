//! Integration tests for production-quality cleanup merge
//!
//! Tests cross-feature interactions after merging:
//! - issue/14-final-validation: Validation scripts and acceptance criteria
//! - issue/13-documentation-consolidation: Documentation structure
//! - issue/12-lexer-vm-clippy-fixes: Clippy warning fixes in lexer and vm
//! - issue/11-compiler-len-zero-fix: Clippy fix in compiler
//!
//! Priority 1: Test clippy-fixed code paths (lexer, vm, compiler)
//! Priority 2: Test interactions between fixed modules
//! Priority 3: Test validation script functionality

use pyrust::execute_python;

#[test]
fn test_lexer_vm_compiler_integration_after_clippy_fixes() {
    // Test that clippy fixes in lexer.rs, vm.rs, and compiler.rs
    // don't break the end-to-end compilation and execution flow
    let code = "x = 42\nprint(x)";
    let result = execute_python(code).expect("Execution should succeed");
    assert_eq!(result, "42\n");
}

#[test]
fn test_compiler_param_is_empty_idiom() {
    // Test that the compiler's !params.is_empty() idiom works correctly
    // This was changed from params.len() > 0 in issue/11-compiler-len-zero-fix

    // Test function with no parameters
    let code_no_params = "def foo():\n    return 42\nx = foo()";
    let result = execute_python(code_no_params);
    assert!(result.is_ok(), "Function with no params should work");

    // Test function with parameters
    let code_with_params = "def add(a, b):\n    return a + b\nx = add(10, 20)\nprint(x)";
    let result = execute_python(code_with_params);
    assert!(result.is_ok(), "Function with params should work");
    assert_eq!(result.unwrap(), "30\n");
}

#[test]
fn test_lexer_pattern_matching_after_clippy_fix() {
    // Test lexer pattern matching after redundant_pattern_matching fix
    // Ensure that the fix didn't break token recognition

    let test_cases = vec![
        ("42", "42"),
        ("x + y", ""), // No output for bare expression with variables
        ("print(123)", "123\n"),
    ];

    for (code, expected) in test_cases {
        let result = execute_python(code);
        if code == "x + y" {
            // This will fail because x and y are undefined
            assert!(result.is_err(), "Should error on undefined variables");
        } else {
            assert_eq!(result.unwrap(), expected);
        }
    }
}

#[test]
fn test_vm_value_copy_semantics_after_clippy_fix() {
    // Test that VM's Value handling works correctly after clone_on_copy fix
    // Value should implement Copy trait, so this tests that the fix is correct

    let code = r#"
x = 10
y = x
x = 20
print(y)
"#;

    let result = execute_python(code).unwrap();
    // y should still be 10, not 20
    assert_eq!(result, "10\n");
}

#[test]
fn test_cross_module_arithmetic_operations() {
    // Test arithmetic operations flowing through lexer -> parser -> compiler -> vm
    // This exercises all the clippy-fixed modules together

    let test_cases = vec![
        ("print(5 + 3)", "8\n"),
        ("print(10 - 4)", "6\n"),
        ("print(6 * 7)", "42\n"),
        ("print(20 / 4)", "5\n"),
        ("print(17 % 5)", "2\n"),
        ("print(15 // 4)", "3\n"),
    ];

    for (code, expected_output) in test_cases {
        let result = execute_python(code)
            .unwrap_or_else(|e| panic!("Execution failed for '{}': {:?}", code, e));
        assert_eq!(result, expected_output, "Output mismatch for '{}'", code);
    }
}

#[test]
fn test_function_call_through_all_stages() {
    // Test function definition and calls through the complete pipeline
    // Tests compiler parameter handling (is_empty() idiom) and VM function metadata

    let code = r#"
def double(n):
    return n * 2

def triple(n):
    return n * 3

x = double(21)
y = triple(14)
print(x)
print(y)
"#;

    let result = execute_python(code).unwrap();
    assert_eq!(result, "42\n42\n");
}

#[test]
fn test_complex_expression_register_allocation() {
    // Test that register allocation works correctly after compiler clippy fixes
    // Complex expressions require multiple registers

    let code = "print((10 + 20) * (30 - 15) / 5)";
    let result = execute_python(code).unwrap();
    // (10 + 20) * (30 - 15) / 5 = 30 * 15 / 5 = 450 / 5 = 90
    assert_eq!(result, "90\n");
}

#[test]
fn test_nested_function_calls_with_registers() {
    // Test nested function calls that stress register allocation
    // and call frame management in VM

    let code = r#"
def add(a, b):
    return a + b

def multiply(x, y):
    return x * y

result = multiply(add(2, 3), add(4, 6))
print(result)
"#;

    let result = execute_python(code).unwrap();
    // multiply(add(2, 3), add(4, 6)) = multiply(5, 10) = 50
    assert_eq!(result, "50\n");
}

#[test]
fn test_variable_shadowing_across_scopes() {
    // Test that variable interning and scope handling work correctly
    // after compiler changes

    let code = r#"
x = 100

def foo():
    x = 200
    return x

y = foo()
print(x)
print(y)
"#;

    let result = execute_python(code).unwrap();
    // Global x should be 100, returned y should be 200
    assert_eq!(result, "100\n200\n");
}

#[test]
fn test_lexer_error_handling_unchanged() {
    // Ensure lexer error handling still works after clippy fixes

    let invalid_code = "123456789012345678901234567890"; // Too large for i64
    let result = execute_python(invalid_code);
    assert!(result.is_err(), "Should error on integer overflow");
}

#[test]
fn test_vm_register_bitmap_operations() {
    // Test that VM register validity bitmap works correctly
    // after clippy fixes (tests the register allocation system)

    let code = r#"
a = 1
b = 2
c = 3
d = 4
e = 5
print(a)
print(b)
print(c)
print(d)
print(e)
"#;

    let result = execute_python(code).unwrap();
    assert_eq!(result, "1\n2\n3\n4\n5\n");
}

#[test]
fn test_zero_parameter_function_edge_case() {
    // Specific test for the is_empty() vs len() > 0 change
    // Tests the exact code path that was modified

    let code = r#"
def get_constant():
    return 42

def get_another():
    return 99

x = get_constant()
y = get_another()
print(x + y)
"#;

    let result = execute_python(code).unwrap();
    assert_eq!(result, "141\n");
}

#[test]
fn test_print_statement_small_string_optimization() {
    // Test VM's SmallString optimization after clippy fixes
    // Tests inline vs heap storage paths

    // Small string (should use inline storage)
    let small_code = "print(42)";
    let result = execute_python(small_code).unwrap();
    assert_eq!(result, "42\n");

    // Large output (should promote to heap)
    let large_code = r#"
print(1000000)
print(2000000)
print(3000000)
print(4000000)
print(5000000)
"#;
    let result = execute_python(large_code).unwrap();
    assert_eq!(result, "1000000\n2000000\n3000000\n4000000\n5000000\n");
}

#[test]
fn test_multiline_code_with_newlines() {
    // Test lexer's newline handling after clippy fixes

    let code = "x = 1\ny = 2\nz = x + y\nprint(z)";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "3\n");
}

#[test]
fn test_unary_minus_operations() {
    // Test unary operations through the pipeline

    let code = r#"
x = 42
y = -x
print(y)
"#;

    let result = execute_python(code).unwrap();
    assert_eq!(result, "-42\n");
}

#[test]
fn test_integration_all_features_combined() {
    // Comprehensive integration test combining all fixed components
    // Tests: lexer patterns, compiler is_empty, VM copy semantics, register allocation

    let code = r#"
def calculate(a, b, c):
    temp1 = a + b
    temp2 = temp1 * c
    return temp2

x = 5
y = 10
z = 3

result1 = calculate(x, y, z)
result2 = calculate(2, 3, 4)

print(result1)
print(result2)
print(result1 + result2)
"#;

    let result = execute_python(code).unwrap();
    // calculate(5, 10, 3) = (5 + 10) * 3 = 45
    // calculate(2, 3, 4) = (2 + 3) * 4 = 20
    // 45 + 20 = 65
    assert_eq!(result, "45\n20\n65\n");
}

#[test]
fn test_division_by_zero_error_handling() {
    // Test that runtime errors are properly handled after VM fixes
    let code = "print(10 / 0)";
    let result = execute_python(code);
    assert!(result.is_err(), "Division by zero should error");
}

#[test]
fn test_modulo_operations() {
    // Test modulo operator through all stages
    let code = "print(17 % 5)";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "2\n");
}

#[test]
fn test_floor_division_operations() {
    // Test floor division operator through all stages
    let code = "print(17 // 5)";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "3\n");
}

#[test]
fn test_parenthesized_expressions() {
    // Test that operator precedence and parentheses work correctly
    let code = "print((2 + 3) * 4)";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "20\n");
}

#[test]
fn test_multiple_assignments_and_prints() {
    // Test mixed statements through the pipeline
    let code = r#"
a = 10
b = 20
c = a + b
print(c)
d = c * 2
print(d)
"#;

    let result = execute_python(code).unwrap();
    assert_eq!(result, "30\n60\n");
}

#[test]
fn test_empty_program() {
    // Edge case: empty program
    let code = "";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_whitespace_only_program() {
    // Edge case: program with only whitespace and newlines
    let code = "\n\n\n";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_single_expression_statement() {
    // Test that expression statement returns value without newline
    let code = "42";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "42");
    assert!(!result.ends_with('\n'));
}

#[test]
fn test_assignment_produces_no_output() {
    // Test that assignment statement produces no output
    let code = "x = 42";
    let result = execute_python(code).unwrap();
    assert_eq!(result, "");
}
