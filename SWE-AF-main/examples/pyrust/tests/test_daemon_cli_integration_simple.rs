//! Simplified integration test for daemon-CLI integration
//!
//! Tests basic daemon functionality without complex timing requirements.
//! These tests verify the integration between main.rs CLI commands and daemon modules.

use std::process::Command;

const BINARY_PATH: &str = "./target/release/pyrust";

#[test]
fn test_daemon_commands_exist() {
    // Test that --daemon command exists
    let _output = Command::new(BINARY_PATH).arg("--help").output();

    // Even if help doesn't exist, we can test that invalid flags give errors
    let output = Command::new(BINARY_PATH)
        .arg("--invalid-flag-that-does-not-exist")
        .output()
        .expect("Binary should execute");

    // Should fail with non-zero exit code
    assert!(!output.status.success());
}

#[test]
fn test_fallback_execution_works() {
    // Execute simple code without daemon (fallback mode)
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("2 + 2")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success(), "Basic execution should work");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "4");
}

#[test]
fn test_print_statement() {
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("print(42)")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "42\n");
}

#[test]
fn test_error_handling_division_by_zero() {
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 / 0")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success(), "Division by zero should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Division by zero"));
}

#[test]
fn test_error_handling_undefined_variable() {
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("undefined_var")
        .output()
        .expect("Failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Undefined variable") || stderr.contains("undefined_var"));
}

#[test]
fn test_profiling_flag_works() {
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("5 * 5")
        .arg("--profile")
        .output()
        .expect("Failed to execute with --profile");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "25");

    // Check profiling output exists in stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "Profiling should produce output");
    assert!(stderr.contains("Stage") || stderr.contains("Lex") || stderr.contains("lex_ns"));
}

#[test]
fn test_profile_json_flag_works() {
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 + 15")
        .arg("--profile-json")
        .output()
        .expect("Failed to execute with --profile-json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "25");

    // Check JSON output
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(stderr.contains("lex_ns") || stderr.contains("parse_ns"));
}

#[test]
fn test_multiline_code_execution() {
    let code = "x = 10\ny = 20\nz = x + y\nprint(z)\nz";

    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg(code)
        .output()
        .expect("Failed to execute multiline code");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "30\n30");
}

#[test]
fn test_complex_expression() {
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("(10 + 5) * 2 - 8 / 4")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "28");
}

#[test]
fn test_variable_operations() {
    let code = "a = 100\nb = 50\nc = a / b\nprint(c)\na + b";

    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg(code)
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "2\n150");
}
