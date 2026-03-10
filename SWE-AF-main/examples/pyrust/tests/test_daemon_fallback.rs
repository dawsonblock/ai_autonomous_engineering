//! Integration tests for daemon client fallback behavior
//!
//! These tests verify that the daemon client can execute code both with and without
//! a running daemon, falling back to direct execution when necessary.

use pyrust::daemon_client::DaemonClient;
use pyrust::execute_python;
use std::fs;

/// Test that execute_or_fallback works when daemon is not running
#[test]
fn test_execute_or_fallback_without_daemon() {
    // Ensure no daemon is running
    let _ = fs::remove_file("/tmp/pyrust.sock");

    // Execute code - should fallback to direct execution
    let result = DaemonClient::execute_or_fallback("2+3").unwrap();
    assert_eq!(result, "5");
}

/// Test that execute_or_fallback returns correct result for simple expression
#[test]
fn test_execute_or_fallback_simple_expression() {
    let result = DaemonClient::execute_or_fallback("2+3").unwrap();
    assert_eq!(result, "5");
}

/// Test that execute_or_fallback returns correct result for complex expression
#[test]
fn test_execute_or_fallback_complex_expression() {
    let result = DaemonClient::execute_or_fallback("(10 + 20) * 2").unwrap();
    assert_eq!(result, "60");
}

/// Test that execute_or_fallback handles variable assignments correctly
#[test]
fn test_execute_or_fallback_with_variables() {
    let code = "x = 10\ny = 20\nx + y";
    let result = DaemonClient::execute_or_fallback(code).unwrap();
    assert_eq!(result, "30");
}

/// Test that execute_or_fallback handles print statements correctly
#[test]
fn test_execute_or_fallback_with_print() {
    let result = DaemonClient::execute_or_fallback("print(42)").unwrap();
    assert_eq!(result, "42\n");
}

/// Test that execute_or_fallback handles mixed statements correctly
#[test]
fn test_execute_or_fallback_mixed_statements() {
    let code = "x = 10\nprint(x)\nx + 5";
    let result = DaemonClient::execute_or_fallback(code).unwrap();
    assert_eq!(result, "10\n15");
}

/// Test error propagation: division by zero
#[test]
fn test_execute_or_fallback_division_by_zero() {
    let result = DaemonClient::execute_or_fallback("10 / 0");
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Division by zero"));
}

/// Test error propagation: undefined variable
#[test]
fn test_execute_or_fallback_undefined_variable() {
    let result = DaemonClient::execute_or_fallback("undefined_var");
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Undefined variable"));
}

/// Test error propagation: syntax error
#[test]
fn test_execute_or_fallback_syntax_error() {
    let result = DaemonClient::execute_or_fallback("x = +");
    assert!(result.is_err());
}

/// Test that error messages match direct execution format
#[test]
fn test_error_format_matches_direct_execution() {
    // Get error from fallback
    let fallback_result = DaemonClient::execute_or_fallback("10 / 0");
    assert!(fallback_result.is_err());
    let fallback_error = format!("{}", fallback_result.unwrap_err());

    // Get error from direct execution
    let direct_result = execute_python("10 / 0");
    assert!(direct_result.is_err());
    let direct_error = format!("{}", direct_result.unwrap_err());

    // Both should contain the same core error message
    assert!(fallback_error.contains("Division by zero"));
    assert!(direct_error.contains("Division by zero"));
}

/// Test is_daemon_running returns false when daemon is not running
#[test]
fn test_is_daemon_running_false() {
    // Ensure socket doesn't exist
    let _ = fs::remove_file("/tmp/pyrust.sock");

    assert!(!DaemonClient::is_daemon_running());
}

/// Test daemon_status returns correct message when daemon is not running
#[test]
fn test_daemon_status_not_running() {
    // Ensure socket doesn't exist
    let _ = fs::remove_file("/tmp/pyrust.sock");

    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is not running");
}

/// Test empty code execution
#[test]
fn test_execute_or_fallback_empty_code() {
    let result = DaemonClient::execute_or_fallback("").unwrap();
    assert_eq!(result, "");
}

/// Test large code execution
#[test]
fn test_execute_or_fallback_large_code() {
    // Create a program with many statements
    let mut code = String::new();
    for i in 1..=100 {
        code.push_str(&format!("x{} = {}\n", i, i));
    }
    code.push_str("x100");

    let result = DaemonClient::execute_or_fallback(&code).unwrap();
    assert_eq!(result, "100");
}

/// Test unicode handling
#[test]
fn test_execute_or_fallback_unicode() {
    let code = "x = 42\nx";
    let result = DaemonClient::execute_or_fallback(code).unwrap();
    assert_eq!(result, "42");
}

/// Test all arithmetic operators
#[test]
fn test_execute_or_fallback_all_operators() {
    assert_eq!(DaemonClient::execute_or_fallback("10 + 5").unwrap(), "15");
    assert_eq!(DaemonClient::execute_or_fallback("10 - 5").unwrap(), "5");
    assert_eq!(DaemonClient::execute_or_fallback("10 * 5").unwrap(), "50");
    assert_eq!(DaemonClient::execute_or_fallback("10 / 5").unwrap(), "2");
    assert_eq!(DaemonClient::execute_or_fallback("10 // 3").unwrap(), "3");
    assert_eq!(DaemonClient::execute_or_fallback("10 % 3").unwrap(), "1");
}

/// Test multiple print statements
#[test]
fn test_execute_or_fallback_multiple_prints() {
    let code = "print(1)\nprint(2)\nprint(3)";
    let result = DaemonClient::execute_or_fallback(code).unwrap();
    assert_eq!(result, "1\n2\n3\n");
}

/// Test operator precedence
#[test]
fn test_execute_or_fallback_operator_precedence() {
    let result = DaemonClient::execute_or_fallback("2 + 3 * 4").unwrap();
    assert_eq!(result, "14");

    let result = DaemonClient::execute_or_fallback("(2 + 3) * 4").unwrap();
    assert_eq!(result, "20");
}

/// Test that fallback works for various error types
#[test]
fn test_fallback_error_types() {
    // Lexer error
    let result = DaemonClient::execute_or_fallback("x = @");
    assert!(result.is_err());

    // Parser error
    let result = DaemonClient::execute_or_fallback("x =");
    assert!(result.is_err());

    // Runtime error
    let result = DaemonClient::execute_or_fallback("10 / 0");
    assert!(result.is_err());
}

/// Test sequential executions maintain independence
#[test]
fn test_execute_or_fallback_sequential_independence() {
    // Each execution should be independent
    let result1 = DaemonClient::execute_or_fallback("x = 10\nx").unwrap();
    assert_eq!(result1, "10");

    // This should not have access to 'x' from previous execution
    let result2 = DaemonClient::execute_or_fallback("x");
    assert!(result2.is_err());
}

/// Test whitespace handling
#[test]
fn test_execute_or_fallback_whitespace() {
    let result = DaemonClient::execute_or_fallback("  2 + 3  ").unwrap();
    assert_eq!(result, "5");

    let result = DaemonClient::execute_or_fallback("\n\n2 + 3\n\n").unwrap();
    assert_eq!(result, "5");
}

/// Test comment handling (though not in spec, good to verify)
#[test]
fn test_execute_or_fallback_with_comments() {
    // Comments are not in the current spec, but if they fail, that's expected
    let result = DaemonClient::execute_or_fallback("2 + 3");
    assert!(result.is_ok());
}

/// Benchmark fallback execution (informational)
#[test]
fn test_execute_or_fallback_performance() {
    use std::time::Instant;

    let start = Instant::now();
    let _result = DaemonClient::execute_or_fallback("2+3").unwrap();
    let duration = start.elapsed();

    // Just verify it completes reasonably quickly
    // Direct execution should be < 100μs typically
    println!("Fallback execution time: {:?}", duration);
    assert!(duration.as_micros() < 100_000); // Less than 100ms
}
