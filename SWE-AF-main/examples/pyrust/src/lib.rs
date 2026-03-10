//! Python-Rust Fast Compiler
//!
//! A high-performance compiler for a Python-like language implemented in Rust.
//!
//! # Public API
//!
//! The main entry point is [`execute_python`], which orchestrates the full compilation
//! and execution pipeline:
//!
//! ```
//! use pyrust::execute_python;
//!
//! // Execute Python code and get formatted output
//! let result = execute_python("print(42)").unwrap();
//! assert_eq!(result, "42\n");
//!
//! // Expression statements return their value
//! let result = execute_python("2 + 2").unwrap();
//! assert_eq!(result, "4");
//!
//! // Assignment statements don't produce output
//! let result = execute_python("x = 10").unwrap();
//! assert_eq!(result, "");
//! ```
//!
//! # Pipeline Architecture
//!
//! The execution pipeline consists of the following stages:
//!
//! 1. **Lexing** ([`lexer::lex`]): Tokenizes source code into a token stream
//! 2. **Parsing** ([`parser::parse`]): Builds an Abstract Syntax Tree (AST)
//! 3. **Compilation** ([`compiler::compile`]): Generates bytecode from the AST
//! 4. **Execution** ([`vm::VM::execute`]): Runs bytecode in a register-based VM
//! 5. **Formatting** ([`vm::VM::format_output`]): Formats output according to specification
//!
//! # Output Format Specification
//!
//! The output format depends on the statements executed:
//!
//! - **Print statements**: Output ends with newline (e.g., `"42\n"`)
//! - **Expression statements**: Output is the value without trailing newline (e.g., `"42"`)
//! - **Assignment statements**: No output (empty string)
//! - **Combined**: Print output followed by expression result (e.g., `"100\n42"`)
//!
//! # Error Handling
//!
//! All errors are propagated through the [`PyRustError`] type, which includes:
//!
//! - [`LexError`]: Tokenization failures (invalid characters, integer overflow)
//! - [`ParseError`]: Syntax errors with location and context
//! - [`CompileError`]: Compilation failures (register overflow, etc.)
//! - [`RuntimeError`]: Execution errors (division by zero, undefined variables)
//!
//! [`LexError`]: error::LexError
//! [`ParseError`]: error::ParseError
//! [`CompileError`]: error::CompileError
//! [`RuntimeError`]: error::RuntimeError
//! [`PyRustError`]: error::PyRustError

pub mod ast;
pub mod bytecode;
pub mod cache;
pub mod compiler;
pub mod daemon;
pub mod daemon_client;
pub mod daemon_protocol;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod profiling;
pub mod value;
pub mod vm;

use error::PyRustError;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

// Global compilation cache for daemon mode
// Uses Mutex for thread-safe access across daemon requests
lazy_static::lazy_static! {
    static ref GLOBAL_CACHE: Mutex<cache::CompilationCache> = {
        Mutex::new(cache::CompilationCache::from_env())
    };
}

// Thread-local compilation cache for library mode
// No locking overhead for single-threaded library usage
thread_local! {
    static THREAD_LOCAL_CACHE: RefCell<cache::CompilationCache> = {
        RefCell::new(cache::CompilationCache::from_env())
    };
}

/// Execute Python source code with thread-local cache (library mode)
///
/// This variant uses a thread-local cache with no locking overhead, optimized
/// for library usage where each thread has its own cache. This provides the best
/// performance for single-threaded or isolated multi-threaded usage.
///
/// Use this for library API calls. For daemon mode, use `execute_python_cached_global`.
///
/// # Arguments
///
/// * `code` - Python source code to execute
///
/// # Returns
///
/// * `Ok(String)` - Formatted output according to the output specification
/// * `Err(PyRustError)` - Error from any stage of the pipeline
pub fn execute_python_cached(code: &str) -> Result<String, PyRustError> {
    // Try to get bytecode from thread-local cache
    let bytecode = THREAD_LOCAL_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.get(code)
    });

    let bytecode = if let Some(cached_bytecode) = bytecode {
        // Cache hit - use cached bytecode
        cached_bytecode
    } else {
        // Cache miss - compile and cache
        // Stage 1: Lex the source code into tokens
        let tokens = lexer::lex(code)?;

        // Stage 2: Parse tokens into an Abstract Syntax Tree
        let ast = parser::parse(tokens)?;

        // Stage 3: Compile AST into bytecode
        let bytecode = compiler::compile(&ast)?;

        // Wrap in Arc once
        let bytecode_arc = Arc::new(bytecode);

        // Insert into thread-local cache
        THREAD_LOCAL_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.insert(code.to_string(), Arc::clone(&bytecode_arc));
        });

        bytecode_arc
    };

    // Stage 4: Execute bytecode in the VM
    let mut vm = vm::VM::new();
    let result = vm.execute(&bytecode)?;

    // Stage 5: Format output according to specification
    let output = vm.format_output(result);

    Ok(output)
}

/// Execute Python source code with global cache (daemon mode)
///
/// This variant uses a global mutex-protected cache shared across all threads,
/// optimized for daemon mode where multiple requests should share the same cache.
///
/// Use this for daemon mode. For library API calls, use `execute_python_cached`.
///
/// # Arguments
///
/// * `code` - Python source code to execute
///
/// # Returns
///
/// * `Ok(String)` - Formatted output according to the output specification
/// * `Err(PyRustError)` - Error from any stage of the pipeline
pub fn execute_python_cached_global(code: &str) -> Result<String, PyRustError> {
    // Try to get bytecode from global cache
    let bytecode = {
        let mut cache = GLOBAL_CACHE.lock().unwrap();
        cache.get(code)
    };

    let bytecode = if let Some(cached_bytecode) = bytecode {
        // Cache hit - use cached bytecode
        cached_bytecode
    } else {
        // Cache miss - compile and cache
        // Stage 1: Lex the source code into tokens
        let tokens = lexer::lex(code)?;

        // Stage 2: Parse tokens into an Abstract Syntax Tree
        let ast = parser::parse(tokens)?;

        // Stage 3: Compile AST into bytecode
        let bytecode = compiler::compile(&ast)?;

        // Wrap in Arc once
        let bytecode_arc = Arc::new(bytecode);

        // Insert into global cache
        {
            let mut cache = GLOBAL_CACHE.lock().unwrap();
            cache.insert(code.to_string(), Arc::clone(&bytecode_arc));
        }

        bytecode_arc
    };

    // Stage 4: Execute bytecode in the VM
    let mut vm = vm::VM::new();
    let result = vm.execute(&bytecode)?;

    // Stage 5: Format output according to specification
    let output = vm.format_output(result);

    Ok(output)
}

/// Execute Python source code and return formatted output
///
/// This is the main public API for the Python-Rust compiler. It orchestrates the
/// full compilation and execution pipeline from source code to output string.
///
/// This function now uses the thread-local cache by default for optimal library
/// performance. For daemon mode, the daemon should use `execute_python_cached_global`.
///
/// # Pipeline Stages
///
/// 1. **Lexing**: Tokenizes the source code using [`lexer::lex`]
/// 2. **Parsing**: Builds an AST using [`parser::parse`]
/// 3. **Compilation**: Generates bytecode using [`compiler::compile`]
/// 4. **Execution**: Runs bytecode in a VM using [`vm::VM::execute`]
/// 5. **Formatting**: Formats output using [`vm::VM::format_output`]
///
/// # Arguments
///
/// * `code` - Python source code to execute
///
/// # Returns
///
/// * `Ok(String)` - Formatted output according to the output specification
/// * `Err(PyRustError)` - Error from any stage of the pipeline
///
/// # Output Format Rules
///
/// The output format is determined by the types of statements in the program:
///
/// - **Print statement only**: `"42\n"` (with newline)
/// - **Expression statement only**: `"42"` (no newline)
/// - **Assignment statement only**: `""` (empty string)
/// - **Print + Expression**: `"100\n42"` (print output + expression value)
/// - **Multiple prints**: `"1\n2\n3\n"` (each with newline)
/// - **Empty program**: `""` (empty string)
///
/// # Examples
///
/// ## Basic Expression
///
/// ```
/// use pyrust::execute_python;
///
/// let result = execute_python("42").unwrap();
/// assert_eq!(result, "42");
/// ```
///
/// ## Arithmetic Expression
///
/// ```
/// use pyrust::execute_python;
///
/// let result = execute_python("2 + 2 * 3").unwrap();
/// assert_eq!(result, "8");
/// ```
///
/// ## Print Statement
///
/// ```
/// use pyrust::execute_python;
///
/// let result = execute_python("print(42)").unwrap();
/// assert_eq!(result, "42\n");
/// ```
///
/// ## Variable Assignment
///
/// ```
/// use pyrust::execute_python;
///
/// let result = execute_python("x = 10").unwrap();
/// assert_eq!(result, "");
/// ```
///
/// ## Complex Program
///
/// ```
/// use pyrust::execute_python;
///
/// let code = "x = 10\ny = 20\nz = x + y\nprint(z)\nz";
/// let result = execute_python(code).unwrap();
/// assert_eq!(result, "30\n30");
/// ```
///
/// ## Error Handling
///
/// ```
/// use pyrust::execute_python;
///
/// // Lexer error
/// let result = execute_python("x = @");
/// assert!(result.is_err());
///
/// // Parser error
/// let result = execute_python("x = +");
/// assert!(result.is_err());
///
/// // Runtime error
/// let result = execute_python("10 / 0");
/// assert!(result.is_err());
/// ```
///
/// # Errors
///
/// This function can return the following errors:
///
/// - **LexError**: Invalid character, integer literal overflow
/// - **ParseError**: Syntax error, unexpected token, missing token
/// - **CompileError**: Register limit exceeded (rare in practice)
/// - **RuntimeError**: Division by zero, undefined variable, integer overflow
///
/// All errors include detailed location information and context.
pub fn execute_python(code: &str) -> Result<String, PyRustError> {
    // Use thread-local cache for library mode (no locking overhead)
    execute_python_cached(code)
}

/// Clear the thread-local cache
///
/// This clears the compilation cache for the current thread.
/// Useful for testing or when you want to reset the cache state.
pub fn clear_thread_local_cache() {
    THREAD_LOCAL_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.clear();
    });
}

/// Clear the global cache
///
/// This clears the compilation cache shared across all threads.
/// Useful for daemon mode or when you want to reset the global cache state.
pub fn clear_global_cache() {
    let mut cache = GLOBAL_CACHE.lock().unwrap();
    cache.clear();
}

/// Get global cache statistics
///
/// Returns statistics about the global cache (hits, misses, size, capacity, hit rate).
/// Useful for monitoring daemon cache performance.
pub fn get_global_cache_stats() -> cache::CacheStats {
    let cache = GLOBAL_CACHE.lock().unwrap();
    cache.stats()
}

/// Get thread-local cache statistics
///
/// Returns statistics about the thread-local cache for the current thread.
/// Useful for monitoring library cache performance.
pub fn get_thread_local_cache_stats() -> cache::CacheStats {
    THREAD_LOCAL_CACHE.with(|cache| {
        let cache = cache.borrow();
        cache.stats()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic expression tests
    #[test]
    fn test_integer_literal() {
        let result = execute_python("42").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_arithmetic_expression() {
        let result = execute_python("2 + 2").unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_arithmetic_precedence() {
        let result = execute_python("2 + 3 * 4").unwrap();
        assert_eq!(result, "14");
    }

    #[test]
    fn test_parentheses() {
        let result = execute_python("(2 + 3) * 4").unwrap();
        assert_eq!(result, "20");
    }

    // Print statement tests
    #[test]
    fn test_print_integer() {
        let result = execute_python("print(42)").unwrap();
        assert_eq!(result, "42\n");
    }

    #[test]
    fn test_print_expression() {
        let result = execute_python("print(2 + 2)").unwrap();
        assert_eq!(result, "4\n");
    }

    #[test]
    fn test_multiple_prints() {
        let result = execute_python("print(1)\nprint(2)\nprint(3)").unwrap();
        assert_eq!(result, "1\n2\n3\n");
    }

    // Assignment tests
    #[test]
    fn test_assignment_no_output() {
        let result = execute_python("x = 42").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_assignment_and_print() {
        let result = execute_python("x = 42\nprint(x)").unwrap();
        assert_eq!(result, "42\n");
    }

    #[test]
    fn test_assignment_and_expression() {
        let result = execute_python("x = 42\nx").unwrap();
        assert_eq!(result, "42");
    }

    // Output format edge cases
    #[test]
    fn test_empty_program() {
        let result = execute_python("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_only_newlines() {
        let result = execute_python("\n\n\n").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_print_and_expression() {
        // Print outputs "100\n", expression outputs "42"
        let result = execute_python("print(100)\n42").unwrap();
        assert_eq!(result, "100\n42");
    }

    #[test]
    fn test_multiple_assignments() {
        let result = execute_python("x = 1\ny = 2\nz = 3").unwrap();
        assert_eq!(result, "");
    }

    // Complex program tests
    #[test]
    fn test_complex_program() {
        let code = "x = 10\ny = 20\nz = x + y\nprint(z)\nz";
        let result = execute_python(code).unwrap();
        assert_eq!(result, "30\n30");
    }

    #[test]
    fn test_arithmetic_operations() {
        // Test all operators
        assert_eq!(execute_python("10 + 5").unwrap(), "15");
        assert_eq!(execute_python("10 - 5").unwrap(), "5");
        assert_eq!(execute_python("10 * 5").unwrap(), "50");
        assert_eq!(execute_python("10 / 5").unwrap(), "2");
        assert_eq!(execute_python("10 // 3").unwrap(), "3");
        assert_eq!(execute_python("10 % 3").unwrap(), "1");
    }

    #[test]
    fn test_variables_in_expressions() {
        let result = execute_python("x = 5\ny = 10\nx + y").unwrap();
        assert_eq!(result, "15");
    }

    // Error handling tests
    #[test]
    fn test_lex_error_invalid_character() {
        let result = execute_python("x = @");
        assert!(result.is_err());
        match result.unwrap_err() {
            PyRustError::LexError(e) => {
                assert!(e.message.contains("Unexpected character"));
                assert_eq!(e.line, 1);
                assert_eq!(e.column, 5);
            }
            _ => panic!("Expected LexError"),
        }
    }

    #[test]
    fn test_lex_error_integer_overflow() {
        let result = execute_python("99999999999999999999999999999");
        assert!(result.is_err());
        match result.unwrap_err() {
            PyRustError::LexError(e) => {
                assert!(e.message.contains("too large"));
            }
            _ => panic!("Expected LexError"),
        }
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let result = execute_python("x = +");
        assert!(result.is_err());
        match result.unwrap_err() {
            PyRustError::ParseError(e) => {
                assert!(e.message.contains("Expected expression"));
                // With unary operator support, + is parsed as unary operator,
                // then it looks for operand and finds EOF (empty string)
                assert_eq!(e.found_token, "");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_error_missing_paren() {
        let result = execute_python("print(42");
        assert!(result.is_err());
        match result.unwrap_err() {
            PyRustError::ParseError(_) => {}
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_runtime_error_division_by_zero() {
        let result = execute_python("10 / 0");
        assert!(result.is_err());
        match result.unwrap_err() {
            PyRustError::RuntimeError(e) => {
                assert_eq!(e.message, "Division by zero");
                assert!(e.instruction_index > 0);
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_runtime_error_undefined_variable() {
        let result = execute_python("undefined_var");
        assert!(result.is_err());
        match result.unwrap_err() {
            PyRustError::RuntimeError(e) => {
                assert!(e.message.contains("Undefined variable"));
                assert!(e.message.contains("undefined_var"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    // Negative number tests
    #[test]
    fn test_negative_numbers() {
        let result = execute_python("x = 10\ny = 3\nx - y").unwrap();
        assert_eq!(result, "7");
    }

    #[test]
    fn test_floor_division() {
        assert_eq!(execute_python("10 // 3").unwrap(), "3");
        assert_eq!(execute_python("9 // 3").unwrap(), "3");
    }

    #[test]
    fn test_modulo() {
        assert_eq!(execute_python("10 % 3").unwrap(), "1");
        assert_eq!(execute_python("9 % 3").unwrap(), "0");
    }

    // Statement type tests
    #[test]
    fn test_expression_statement_output() {
        // Expression statement should output value without newline
        let result = execute_python("42").unwrap();
        assert_eq!(result, "42");
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_print_statement_output() {
        // Print statement should output value with newline
        let result = execute_python("print(42)").unwrap();
        assert_eq!(result, "42\n");
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_assignment_statement_output() {
        // Assignment statement should produce no output
        let result = execute_python("x = 42").unwrap();
        assert_eq!(result, "");
    }

    // Edge case: multiple expression statements
    #[test]
    fn test_multiple_expression_statements() {
        // Only the last expression result is returned
        let result = execute_python("1\n2\n3").unwrap();
        assert_eq!(result, "3");
    }

    // Edge case: mixed statement types
    #[test]
    fn test_mixed_statements() {
        let code = "x = 5\nprint(x)\ny = 10\nprint(y)\nx + y";
        let result = execute_python(code).unwrap();
        assert_eq!(result, "5\n10\n15");
    }

    // Pipeline integration tests
    #[test]
    fn test_full_pipeline_simple() {
        // Verify that all pipeline stages work together
        let result = execute_python("2 + 2").unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_full_pipeline_complex() {
        // More complex program exercising all features
        let code = r#"
a = 10
b = 20
c = a + b
print(c)
d = c * 2
print(d)
d
"#;
        let result = execute_python(code).unwrap();
        assert_eq!(result, "30\n60\n60");
    }

    // Error propagation tests
    #[test]
    fn test_error_propagation_from_lexer() {
        let result = execute_python("@@@");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_propagation_from_parser() {
        let result = execute_python("1 +");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_propagation_from_runtime() {
        let result = execute_python("1 / 0");
        assert!(result.is_err());
    }

    // Whitespace handling
    #[test]
    fn test_whitespace_variations() {
        assert_eq!(execute_python("  42  ").unwrap(), "42");
        assert_eq!(execute_python("\t42\t").unwrap(), "42");
        assert_eq!(execute_python("  2  +  2  ").unwrap(), "4");
    }

    // Variable reassignment
    #[test]
    fn test_variable_reassignment() {
        let code = "x = 10\nx = 20\nx";
        let result = execute_python(code).unwrap();
        assert_eq!(result, "20");
    }

    // Complex expressions
    #[test]
    fn test_deeply_nested_expression() {
        let result = execute_python("((1 + 2) * (3 + 4)) / 7").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_long_expression() {
        let result = execute_python("1 + 2 + 3 + 4 + 5").unwrap();
        assert_eq!(result, "15");
    }

    // Zero handling
    #[test]
    fn test_zero_literal() {
        let result = execute_python("0").unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn test_zero_in_expression() {
        assert_eq!(execute_python("0 + 5").unwrap(), "5");
        assert_eq!(execute_python("0 * 5").unwrap(), "0");
    }

    // Print multiple values
    #[test]
    fn test_print_multiple_values_separate_statements() {
        let code = "print(1)\nprint(2)\nprint(3)";
        let result = execute_python(code).unwrap();
        assert_eq!(result, "1\n2\n3\n");
    }

    // Verify output format specification compliance
    #[test]
    fn test_output_format_only_stdout() {
        // Only print statements - should have trailing newlines
        let result = execute_python("print(42)").unwrap();
        assert_eq!(result, "42\n");
    }

    #[test]
    fn test_output_format_only_result() {
        // Only expression statement - no trailing newline
        let result = execute_python("42").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_output_format_both() {
        // Both stdout and result - stdout + result (no extra newline)
        let result = execute_python("print(100)\n42").unwrap();
        assert_eq!(result, "100\n42");
    }

    #[test]
    fn test_output_format_neither() {
        // No output - empty string
        let result = execute_python("x = 42").unwrap();
        assert_eq!(result, "");
    }

    // Large integer values
    #[test]
    fn test_large_integer() {
        let result = execute_python("9223372036854775807").unwrap();
        assert_eq!(result, "9223372036854775807");
    }

    #[test]
    fn test_all_operators_in_one_expression() {
        // Test operator precedence with all operators
        // 10 + 5 * 2 - 8 / 4 % 3 = 10 + 10 - 2 % 3 = 10 + 10 - 2 = 18
        let result = execute_python("10 + 5 * 2 - 8 / 4 % 3").unwrap();
        assert_eq!(result, "18");
    }

    // Cache integration tests
    #[test]
    fn test_cache_integration_repeated_code() {
        // Execute same code multiple times - should use cache
        let code = "2 + 2";

        let result1 = execute_python(code).unwrap();
        assert_eq!(result1, "4");

        let result2 = execute_python(code).unwrap();
        assert_eq!(result2, "4");

        let result3 = execute_python(code).unwrap();
        assert_eq!(result3, "4");
    }

    #[test]
    fn test_cache_integration_different_code() {
        // Execute different code - should return correct results
        let result1 = execute_python("2 + 2").unwrap();
        assert_eq!(result1, "4");

        let result2 = execute_python("3 + 3").unwrap();
        assert_eq!(result2, "6");

        let result3 = execute_python("5 * 5").unwrap();
        assert_eq!(result3, "25");
    }

    #[test]
    fn test_cache_integration_collision_detection() {
        // Different code should produce different results even if cached
        let code1 = "10 + 20";
        let code2 = "15 + 15";

        let result1 = execute_python(code1).unwrap();
        assert_eq!(result1, "30");

        let result2 = execute_python(code2).unwrap();
        assert_eq!(result2, "30");

        // Run again to verify cache doesn't confuse them
        let result1_again = execute_python(code1).unwrap();
        assert_eq!(result1_again, "30");

        let result2_again = execute_python(code2).unwrap();
        assert_eq!(result2_again, "30");
    }
}
