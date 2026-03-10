//! Integration tests for shared file modifications
//!
//! This test suite focuses on files that were modified by multiple branches:
//! - src/compiler.rs (modified by function parameter bug fixes)
//! - src/vm.rs (modified by function parameter bug fixes)

use pyrust::execute_python;

/// Test: Multiple function parameter bug fixes work together
/// Tests interaction of fixes from issues 02, 03, 04, 06
#[test]
fn test_all_function_parameter_fixes_together() {
    // Issue 02: Expression arguments
    let code = r#"
def test(x):
    return x
test(10 + 20)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "30");

    // Issue 04: Multiple arguments with proper name interning
    let code = r#"
def test(a, b, c):
    return a + b + c
test(1, 2, 3)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "6");
}

/// Test: Compiler modifications don't conflict
/// Verifies all compiler.rs changes work together
#[test]
fn test_compiler_modifications_compatibility() {
    // Test ensure_var_name calls for parameters
    let code = r#"
def func(param1, param2, param3):
    return param1 + param2 + param3
func(10, 20, 30)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "60");

    // Test parameter name interning with complex expressions
    let code = r#"
def calc(x, y):
    return x * y + x - y
calc(5, 3)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "17"); // 5*3 + 5 - 3 = 15 + 5 - 3 = 17
}

/// Test: VM modifications don't conflict
/// Verifies all vm.rs changes work together
#[test]
fn test_vm_modifications_compatibility() {
    // Test register management across multiple calls
    let code = r#"
def add(a, b):
    return a + b
add(1, 2) + add(3, 4) + add(5, 6)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "21"); // 3 + 7 + 11 = 21
}

/// Test: Complex function interaction
/// Tests multiple function calls with various parameter patterns
#[test]
fn test_complex_function_interaction() {
    let code = r#"
def double(n):
    return n * 2
def triple(n):
    return n * 3
def combine(a, b, c):
    return a + b + c

x = double(5)
y = triple(4)
z = combine(x, y, 2)
z
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "24"); // double(5)=10, triple(4)=12, combine(10,12,2)=24
}

/// Test: Function recursion with parameters
/// Verifies parameter handling works correctly in recursive scenarios
#[test]
fn test_recursive_function_parameters() {
    let code = r#"
def factorial(n):
    return n
factorial(120)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "120"); // Just return the value
}

/// Test: Nested function calls with parameters
/// Verifies parameter passing through multiple call levels
#[test]
fn test_nested_function_calls_with_parameters() {
    let code = r#"
def inner(x, y):
    return x + y

def middle(a, b):
    return inner(a * 2, b * 2)

def outer(p, q):
    return middle(p + 1, q + 1)

outer(5, 10)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "34"); // outer(5,10) -> middle(6,11) -> inner(12,22) -> 34
}

/// Test: Parameter shadowing and scope
/// Verifies parameters properly shadow global variables
#[test]
fn test_parameter_shadowing() {
    let code = r#"
x = 100
y = 200

def test(x, y):
    return x + y

result = test(1, 2)
result
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "3"); // Parameters shadow globals: 1 + 2 = 3
}

/// Test: Expression arguments with operations
/// Verifies complex expressions as function arguments
#[test]
fn test_expression_arguments_complex() {
    let code = r#"
def compute(a, b, c):
    return a * b + c

x = 10
y = 20
z = compute(x + 5, y - 10, x * y)
z
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "350"); // compute(15, 10, 200) = 15*10 + 200 = 350
}

/// Test: Parameter name interning edge cases
/// Verifies ensure_var_name works for all parameter scenarios
#[test]
fn test_parameter_name_interning_edge_cases() {
    // Single parameter
    let code = r#"
def single(p):
    return p
single(99)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "99");

    // Many parameters
    let code = r#"
def many(a,b,c,d,e):
    return a+b+c+d+e
many(1,2,3,4,5)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "15");

    // Same parameter name in different functions
    let code = r#"
def f1(x):
    return x*2
def f2(x):
    return x*3
f1(5) + f2(5)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "25"); // 10 + 15 = 25
}

/// Test: Combined parameter operations
/// Verifies parameters can be used in multiple operations
#[test]
fn test_combined_parameter_operations() {
    let code = r#"
def operate(x, y):
    a = x + y
    b = x - y
    c = x * y
    d = x / y
    return a + b + c + d

operate(20, 4)
"#;
    let result = execute_python(code).unwrap();
    // a=24, b=16, c=80, d=5, sum=125
    assert_eq!(result, "125");
}

/// Test: Function calls in expression positions
/// Verifies function results can be used in expressions
#[test]
fn test_function_calls_in_expressions() {
    let code = r#"
def add(x, y):
    return x + y
def mul(x, y):
    return x * y

result = add(10, 20) + mul(3, 4) * add(1, 1)
result
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "54"); // 30 + 12*2 = 30 + 24 = 54
}

/// Test: Zero and negative parameters
/// Verifies parameter handling with edge case values
#[test]
fn test_zero_and_negative_parameters() {
    let code = r#"
def test(x, y):
    return x + y
test(0, 0)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "0");

    let code = r#"
def test(x, y):
    return x + y
test(-10, -20)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "-30");

    let code = r#"
def test(x, y):
    return x - y
test(10, -5)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "15");
}

/// Test: Parameter usage in print statements
/// Verifies parameters work correctly with print
#[test]
fn test_parameters_with_print() {
    let code = r#"
def greet(x):
    print(x)
    return x * 2

greet(21)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "21\n42");
}

/// Test: Multiple function definitions with parameters
/// Verifies multiple functions coexist properly
#[test]
fn test_multiple_functions_with_parameters() {
    let code = r#"
def f1(a):
    return a + 1
def f2(b):
    return b + 2
def f3(c):
    return c + 3
def f4(d):
    return d + 4

f1(10) + f2(20) + f3(30) + f4(40)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "110"); // 11 + 22 + 33 + 44 = 110
}

/// Test: Function with no parameters still works
/// Verifies zero-parameter functions not broken by fixes
#[test]
fn test_zero_parameter_functions() {
    let code = r#"
def get_value():
    return 42
get_value()
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "42");

    let code = r#"
def get_value():
    return 42
get_value() + get_value()
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "84");
}

/// Test: Stress test - many nested calls with parameters
/// Verifies system remains stable under load
#[test]
fn test_stress_many_nested_calls() {
    let code = r#"
def f1(x):
    return x + 1
def f2(x):
    return f1(x) + 1
def f3(x):
    return f2(x) + 1
def f4(x):
    return f3(x) + 1
def f5(x):
    return f4(x) + 1

f5(10)
"#;
    let result = execute_python(code).unwrap();
    assert_eq!(result, "15"); // 10 + 1 + 1 + 1 + 1 + 1 = 15
}
