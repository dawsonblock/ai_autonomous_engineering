//! Integration tests for vm.rs conflict resolution
//!
//! Tests the interaction between:
//! - issue/03-vm-dead-code-removal: Removed body_len field from FunctionMetadata
//! - issue/06-code-formatting: Multi-line formatting
//!
//! Priority: HIGH - This is a conflict resolution area
//!
//! Key risks:
//! - DefineFunction instruction matching with body_len: _ pattern
//! - FunctionMetadata initialization without body_len field
//! - Function calls and execution still work correctly
//! - Multi-line formatting in function definition blocks

use pyrust::bytecode::{Bytecode, CompilerMetadata, Instruction};
use pyrust::execute_python;
use pyrust::vm::VM;

#[test]
fn test_vm_function_definition_without_body_len() {
    // Test that DefineFunction works without using body_len field
    let code = r#"
def add(a, b):
    return a + b

result = add(2, 3)
"#;

    let result = execute_python(code);
    assert!(
        result.is_ok(),
        "Function definition should work: {:?}",
        result
    );
    assert_eq!(result.unwrap(), "5");
}

#[test]
fn test_vm_function_metadata_no_body_len_field() {
    // Test that FunctionMetadata is created without body_len field
    // This directly tests the conflict resolution
    let code = r#"
def simple():
    return 42

simple()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_vm_multiple_functions_without_body_len() {
    // Test multiple function definitions (stress test body_len removal)
    let code = r#"
def f1():
    return 1

def f2():
    return 2

def f3():
    return 3

f1() + f2() + f3()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "6");
}

#[test]
fn test_vm_nested_function_calls_without_body_len() {
    // Test nested function calls work without body_len
    let code = r#"
def outer(x):
    def inner(y):
        return y * 2
    return inner(x) + 1

outer(5)
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "11");
}

#[test]
fn test_vm_recursive_function_without_body_len() {
    // Test recursive function works (requires proper function metadata)
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

factorial(5)
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "120");
}

#[test]
fn test_vm_function_with_params_no_body_len() {
    // Test function with multiple parameters
    let code = r#"
def multiply(a, b, c):
    return a * b * c

multiply(2, 3, 4)
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "24");
}

#[test]
fn test_vm_function_overwriting_without_body_len() {
    // Test function redefinition (metadata replacement)
    let code = r#"
def f():
    return 1

f()  # First definition

def f():
    return 2

f()  # Second definition
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_vm_bytecode_define_function_pattern_match() {
    // Test direct bytecode execution with DefineFunction instruction
    // This validates the body_len: _ pattern matching in the conflict resolution

    let var_names = vec!["test_func".to_string()];
    let instructions = vec![
        Instruction::DefineFunction {
            name_index: 0,
            param_count: 0,
            body_start: 2,
            body_len: 1, // This value should be ignored by VM
            max_register_used: 0,
        },
        Instruction::Halt,
        Instruction::Return {
            has_value: false,
            src_reg: None,
        },
    ];

    let bytecode = Bytecode {
        instructions,
        constants: vec![],
        var_names,
        var_ids: vec![0],
        metadata: CompilerMetadata {
            max_register_used: 0,
        },
    };

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    // Should succeed even though body_len is provided but ignored
    assert!(result.is_ok(), "DefineFunction should ignore body_len");
}

#[test]
fn test_vm_function_call_with_body_len_ignored() {
    // Test that function calls work when body_len is in bytecode but ignored
    let var_names = vec!["add".to_string()];
    let instructions = vec![
        Instruction::DefineFunction {
            name_index: 0,
            param_count: 2,
            body_start: 2,
            body_len: 1, // Ignored by VM
            max_register_used: 1,
        },
        Instruction::Halt,
        Instruction::Return {
            has_value: false,
            src_reg: None,
        },
    ];

    let bytecode = Bytecode {
        instructions,
        constants: vec![],
        var_names,
        var_ids: vec![0],
        metadata: CompilerMetadata {
            max_register_used: 1,
        },
    };

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_ok());
}

#[test]
fn test_vm_empty_function_body_without_body_len() {
    // Test function with empty body (edge case)
    let code = r#"
def empty():
    pass

empty()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
}

#[test]
fn test_vm_function_with_print_no_body_len() {
    // Test function with print statement
    let code = r#"
def greet():
    print("hello")

greet()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello\n");
}

#[test]
fn test_vm_function_with_locals_no_body_len() {
    // Test function with local variables
    let code = r#"
def calc():
    x = 10
    y = 20
    return x + y

calc()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "30");
}

#[test]
fn test_vm_function_name_index_validation() {
    // Test that name_index validation still works without body_len
    let var_names = vec![];
    let instructions = vec![
        Instruction::DefineFunction {
            name_index: 99, // Invalid index
            param_count: 0,
            body_start: 2,
            body_len: 1,
            max_register_used: 0,
        },
        Instruction::Halt,
    ];

    let bytecode = Bytecode {
        instructions,
        constants: vec![],
        var_names,
        var_ids: vec![],
        metadata: CompilerMetadata {
            max_register_used: 0,
        },
    };

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    // Should error on invalid name_index
    assert!(result.is_err());
}

#[test]
fn test_vm_max_register_used_without_body_len() {
    // Test that max_register_used is properly handled in FunctionMetadata
    let code = r#"
def uses_registers():
    a = 1
    b = 2
    c = 3
    d = 4
    e = 5
    return a + b + c + d + e

uses_registers()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "15");
}

#[test]
fn test_vm_function_with_conditionals_no_body_len() {
    // Test function with control flow (longer body)
    let code = r#"
def abs_value(x):
    if x < 0:
        return -x
    else:
        return x

abs_value(-42)
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_vm_function_call_stack_without_body_len() {
    // Test deep call stack (ensures function metadata works correctly)
    let code = r#"
def a():
    return b()

def b():
    return c()

def c():
    return 123

a()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "123");
}

#[test]
fn test_vm_formatting_in_function_blocks() {
    // Test multi-line formatting in function definitions
    // This validates the conflict resolution with issue/06
    let code = r#"
def complex_function(
    param1,
    param2,
    param3
):
    result = param1 + param2 + param3
    return result

complex_function(1, 2, 3)
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "6");
}

#[test]
fn test_vm_function_return_values_without_body_len() {
    // Test various return value scenarios
    let test_cases = vec![
        ("def f():\n    return 0\nf()", "0"),
        ("def f():\n    return -1\nf()", "-1"),
        ("def f():\n    return 999\nf()", "999"),
        ("def f():\n    return 2+2\nf()", "4"),
    ];

    for (code, expected) in test_cases {
        let result = execute_python(code);
        assert!(result.is_ok(), "Failed for code: {}", code);
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn test_vm_function_without_return_no_body_len() {
    // Test function without explicit return
    let code = r#"
def no_return():
    x = 1

no_return()
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
}

#[test]
fn test_vm_multiple_calls_same_function_no_body_len() {
    // Test multiple calls to same function
    let code = r#"
def square(x):
    return x * x

square(2) + square(3) + square(4)
"#;

    let result = execute_python(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "29"); // 4 + 9 + 16
}
