//! Integration QA: Compiler ↔ VM Interaction Tests
//!
//! Tests cross-feature interactions between compiler.rs and vm.rs modifications
//! from multiple bug fix branches (issues 02-07)
//!
//! PRIORITY 1: Test function parameter evaluation order (issue 02)
//! PRIORITY 2: Test function forward reference validation (issue 03)
//! PRIORITY 3: Test function parameter naming in bytecode (issue 04)
//! PRIORITY 4: Test function return value handling (issue 05)
//! PRIORITY 5: Test function early return (issue 06)
//! PRIORITY 6: Test negative number parsing through full pipeline (issue 07)

use pyrust::execute_python;

/// Test that function call arguments are evaluated left-to-right
/// and passed correctly from compiler to VM
#[test]
fn test_compiler_vm_function_arg_evaluation_order() {
    // Test case from issue 02: expression arguments must be evaluated left-to-right
    let code = r#"
def foo(a, b, c):
    return a + b * c

x = 10
result = foo(x, x + 5, x + 10)
result
"#;
    let result = execute_python(code).expect("Execution failed");
    // foo(10, 15, 20) = 10 + 15*20 = 10 + 300 = 310
    assert_eq!(result, "310", "Left-to-right evaluation failed");
}

/// Test that VM properly handles function calls with expression arguments
#[test]
fn test_compiler_vm_expression_args_integration() {
    let code = r#"
def add(a, b):
    return a + b

x = 5
y = 10
add(x * 2, y / 2)
"#;
    let result = execute_python(code).expect("Execution failed");
    // add(10, 5) = 15
    assert_eq!(result, "15");
}

/// Test that calling function before definition produces compile error
/// (compiler validation caught before VM execution)
#[test]
fn test_compiler_catches_forward_reference_before_vm() {
    let code = r#"
x = foo(10)
def foo(n):
    return n * 2
x
"#;
    let result = execute_python(code);
    assert!(result.is_err(), "Should fail at compile time, not runtime");

    // Verify it's an appropriate error
    match result {
        Err(e) => {
            let err_str = format!("{:?}", e);
            // Should be CompileError with "undefined" or "defined later" message
            assert!(
                err_str.contains("undefined")
                    || err_str.contains("Undefined")
                    || err_str.contains("forward")
                    || err_str.contains("defined later"),
                "Expected forward reference error, got: {}",
                err_str
            );
        }
        Ok(_) => panic!("Should have failed"),
    }
}

/// Test that VM correctly resolves parameter names from compiler's var_names pool
#[test]
fn test_compiler_vm_parameter_name_resolution() {
    let code = r#"
def calculate(first, second, third):
    return first + second + third

calculate(10, 20, 30)
"#;
    let result = execute_python(code).expect("Parameter resolution failed");
    assert_eq!(result, "60");
}

/// Test complex parameter usage across compiler and VM
#[test]
fn test_compiler_vm_multiple_param_usage() {
    let code = r#"
def compute(a, b, c):
    x = a + b
    y = b + c
    z = x + y
    return z

compute(5, 10, 15)
"#;
    let result = execute_python(code).expect("Multi-param computation failed");
    // x = 15, y = 25, z = 40
    assert_eq!(result, "40");
}

/// Test that VM properly handles return instruction from compiler
#[test]
fn test_compiler_vm_return_value_propagation() {
    let code = r#"
def calculate(x, y):
    result = x * y + 10
    return result

calculate(5, 6)
"#;
    let result = execute_python(code).expect("Return value propagation failed");
    assert_eq!(result, "40"); // 5*6 + 10 = 40
}

/// Test early return paths compiled correctly and executed by VM
#[test]
fn test_compiler_vm_early_return_execution() {
    // Test simple early return
    let code_simple = r#"
def early():
    return 42
    x = 100
    return x

early()
"#;
    let result = execute_python(code_simple).expect("Early return failed");
    assert_eq!(result, "42");
}

/// Test negative number parsing through lexer→parser→compiler→VM pipeline
#[test]
fn test_compiler_vm_negative_number_pipeline() {
    let code = r#"
def negate(x):
    return -x

result = negate(42)
result
"#;
    let result = execute_python(code).expect("Negative number handling failed");
    assert_eq!(result, "-42");
}

/// Test negative function parameters through full pipeline
#[test]
fn test_compiler_vm_negative_parameters() {
    let code = r#"
def sub(a, b):
    return a - b

sub(10, 40)
"#;
    let result = execute_python(code).expect("Negative result failed");
    assert_eq!(result, "-30");
}

/// Test that negative number literals are parsed and executed correctly
#[test]
fn test_compiler_vm_negative_literal_expression() {
    // Test unary minus operator
    let code = "-42";
    let result = execute_python(code).expect("Negative literal failed");
    assert_eq!(result, "-42");
}

/// Test complex negative number arithmetic
#[test]
fn test_compiler_vm_negative_arithmetic_complex() {
    let code = r#"
x = 10
y = 40
result = x - y
result
"#;
    let result = execute_python(code).expect("Negative arithmetic failed");
    assert_eq!(result, "-30");
}

/// Test function returning negative value used in expression
#[test]
fn test_compiler_vm_function_negative_in_expression() {
    let code = r#"
def get_negative():
    return -100

x = get_negative()
y = x + 50
y
"#;
    let result = execute_python(code).expect("Function negative in expression failed");
    assert_eq!(result, "-50");
}

/// Stress test: Multiple functions with various parameter patterns
#[test]
fn test_compiler_vm_multiple_functions_stress() {
    let code = r#"
def add(a, b):
    return a + b

def mul(x, y):
    return x * y

def combo(p, q, r):
    temp1 = add(p, q)
    temp2 = mul(temp1, r)
    return temp2

combo(3, 7, 5)
"#;
    let result = execute_python(code).expect("Multiple functions failed");
    // add(3,7)=10, mul(10,5)=50
    assert_eq!(result, "50");
}

/// Test that all bug fixes work together in one program
#[test]
fn test_compiler_vm_all_fixes_combined() {
    let code_simple = r#"
def process(a, b, c):
    x = a + b
    y = x - c
    return y

val1 = process(10, 20, 40)
val2 = process(5, 15, 10)
val1 + val2
"#;
    let result = execute_python(code_simple).expect("Combined fixes failed");
    // val1 = -10, val2 = 10, sum = 0
    assert_eq!(result, "0");
}
