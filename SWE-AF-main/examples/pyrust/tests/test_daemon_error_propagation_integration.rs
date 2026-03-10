//! Integration test for daemon error propagation
//!
//! Tests that errors are correctly propagated from daemon to client
//! and match the format of direct execution errors.
//!
//! Priority: High (tests AC2.5 - error propagation)
//!
//! Verifies:
//! - Daemon returns identical error format as direct execution
//! - Division by zero errors
//! - Undefined variable errors
//! - Syntax errors
//! - Error propagation through daemon_protocol → daemon → daemon_client → main

use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

const BINARY_PATH: &str = "./target/release/pyrust";
const SOCKET_PATH: &str = "/tmp/pyrust.sock";
const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

fn cleanup() {
    if std::path::Path::new(PID_FILE_PATH).exists() {
        if let Ok(pid_str) = fs::read_to_string(PID_FILE_PATH) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
    }

    let _ = fs::remove_file(SOCKET_PATH);
    let _ = fs::remove_file(PID_FILE_PATH);
    thread::sleep(Duration::from_millis(100));
}

fn get_error_output(code: &str, with_daemon: bool) -> String {
    if with_daemon {
        // Execute through daemon
        let output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(code)
            .output()
            .expect("Failed to execute through daemon");

        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        // Execute in fallback mode (no daemon)
        let output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(code)
            .output()
            .expect("Failed to execute in fallback");

        String::from_utf8_lossy(&output.stderr).to_string()
    }
}

#[test]
fn test_division_by_zero_error_matches() {
    cleanup();

    // Get error from fallback mode
    let fallback_error = get_error_output("10 / 0", false);

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Get error from daemon mode
    let daemon_error = get_error_output("10 / 0", true);

    // Errors should match
    assert_eq!(fallback_error.trim(), daemon_error.trim(),
               "Division by zero error should match between daemon and fallback.\nFallback: {}\nDaemon: {}",
               fallback_error, daemon_error);

    // Verify error contains expected message
    assert!(
        daemon_error.contains("Division by zero"),
        "Error should mention division by zero: {}",
        daemon_error
    );

    cleanup();
}

#[test]
fn test_undefined_variable_error_matches() {
    cleanup();

    let test_code = "undefined_variable";

    // Get error from fallback mode
    let fallback_error = get_error_output(test_code, false);

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Get error from daemon mode
    let daemon_error = get_error_output(test_code, true);

    // Errors should match
    assert_eq!(
        fallback_error.trim(),
        daemon_error.trim(),
        "Undefined variable error should match.\nFallback: {}\nDaemon: {}",
        fallback_error,
        daemon_error
    );

    // Verify error contains expected message
    assert!(
        daemon_error.contains("Undefined variable") || daemon_error.contains("undefined_variable"),
        "Error should mention undefined variable: {}",
        daemon_error
    );

    cleanup();
}

#[test]
fn test_syntax_error_matches() {
    cleanup();

    let test_code = "x = +";

    // Get error from fallback mode
    let fallback_error = get_error_output(test_code, false);

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Get error from daemon mode
    let daemon_error = get_error_output(test_code, true);

    // Errors should match
    assert_eq!(
        fallback_error.trim(),
        daemon_error.trim(),
        "Syntax error should match.\nFallback: {}\nDaemon: {}",
        fallback_error,
        daemon_error
    );

    // Verify error contains expected message
    assert!(
        daemon_error.contains("Expected expression") || daemon_error.contains("ParseError"),
        "Error should mention parse error: {}",
        daemon_error
    );

    cleanup();
}

#[test]
fn test_lexer_error_matches() {
    cleanup();

    let test_code = "x = @";

    // Get error from fallback mode
    let fallback_error = get_error_output(test_code, false);

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Get error from daemon mode
    let daemon_error = get_error_output(test_code, true);

    // Errors should match
    assert_eq!(
        fallback_error.trim(),
        daemon_error.trim(),
        "Lexer error should match.\nFallback: {}\nDaemon: {}",
        fallback_error,
        daemon_error
    );

    // Verify error contains expected message
    assert!(
        daemon_error.contains("Unexpected character") || daemon_error.contains("LexError"),
        "Error should mention lex error: {}",
        daemon_error
    );

    cleanup();
}

#[test]
fn test_error_exit_codes_match() {
    cleanup();

    let test_cases = vec![
        "10 / 0",        // Runtime error
        "undefined_var", // Runtime error
        "x = +",         // Parse error
        "x = @",         // Lex error
    ];

    for test_code in test_cases {
        // Get exit code from fallback mode
        let fallback_output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(test_code)
            .output()
            .expect("Failed to execute in fallback");

        let fallback_code = fallback_output.status.code().unwrap_or(0);

        // Start daemon
        let _ = Command::new(BINARY_PATH).arg("--daemon").output();

        thread::sleep(Duration::from_millis(300));

        // Get exit code from daemon mode
        let daemon_output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(test_code)
            .output()
            .expect("Failed to execute through daemon");

        let daemon_code = daemon_output.status.code().unwrap_or(0);

        // Stop daemon
        let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

        thread::sleep(Duration::from_millis(200));

        // Exit codes should match
        assert_eq!(
            fallback_code, daemon_code,
            "Exit codes should match for '{}'. Fallback: {}, Daemon: {}",
            test_code, fallback_code, daemon_code
        );

        // Both should be non-zero (error)
        assert_ne!(
            daemon_code, 0,
            "Error exit code should be non-zero for '{}'",
            test_code
        );
    }

    cleanup();
}

#[test]
fn test_multiple_errors_in_sequence() {
    cleanup();

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Execute multiple error-producing codes in sequence
    let error_codes = vec![
        ("10 / 0", "Division by zero"),
        ("x", "Undefined variable"),
        ("1 / 0", "Division by zero"),
        ("y + z", "Undefined variable"),
    ];

    for (code, expected_msg) in error_codes {
        let output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(code)
            .output()
            .expect("Failed to execute");

        assert!(!output.status.success(), "Code '{}' should fail", code);

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_msg),
            "Error for '{}' should contain '{}', got: {}",
            code,
            expected_msg,
            stderr
        );
    }

    // Verify daemon still works after errors
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("2 + 2")
        .output()
        .expect("Failed to execute after errors");

    assert!(
        output.status.success(),
        "Daemon should still work after handling errors"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "4");

    cleanup();
}

#[test]
fn test_error_with_multiline_code() {
    cleanup();

    let multiline_error_code = "x = 10\ny = 20\nz = x / 0\nprint(z)";

    // Get error from fallback
    let fallback_error = get_error_output(multiline_error_code, false);

    // Start daemon
    let _ = Command::new(BINARY_PATH).arg("--daemon").output();

    thread::sleep(Duration::from_millis(300));

    // Get error from daemon
    let daemon_error = get_error_output(multiline_error_code, true);

    // Errors should match
    assert_eq!(
        fallback_error.trim(),
        daemon_error.trim(),
        "Multiline error should match.\nFallback: {}\nDaemon: {}",
        fallback_error,
        daemon_error
    );

    assert!(
        daemon_error.contains("Division by zero"),
        "Error should mention division by zero: {}",
        daemon_error
    );

    cleanup();
}

#[test]
fn test_daemon_survives_malformed_requests() {
    cleanup();

    // Start daemon
    let _ = Command::new(BINARY_PATH).arg("--daemon").output();

    thread::sleep(Duration::from_millis(300));

    // Send various malformed/error requests
    for _ in 0..5 {
        let _ = Command::new(BINARY_PATH).arg("-c").arg("@@@").output();
    }

    // Daemon should still work
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("42")
        .output()
        .expect("Failed to execute");

    assert!(
        output.status.success(),
        "Daemon should survive malformed requests"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");

    cleanup();
}
