//! Integration test for daemon and profiling mode separation
//!
//! Tests that profiling mode bypasses daemon and uses direct execution.
//! This is critical because profiling needs precise timing without daemon overhead.
//!
//! Tests the interaction between:
//! - CLI profiling flags (main.rs)
//! - Profiling module (profiling.rs)
//! - Daemon bypass logic (main.rs lines 62-82)
//!
//! Priority: High (tests architectural constraint - profiling must bypass daemon)

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

#[test]
fn test_profiling_bypasses_daemon() {
    cleanup();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success());
    thread::sleep(Duration::from_millis(300));
    assert!(std::path::Path::new(SOCKET_PATH).exists());

    // Execute with --profile flag (should bypass daemon)
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("2 + 2")
        .arg("--profile")
        .output()
        .expect("Failed to execute with profiling");

    assert!(output.status.success(), "Profiling execution failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check output is correct
    assert_eq!(stdout.trim(), "4", "Profiling output incorrect");

    // Check profiling output is present in stderr
    assert!(
        stderr.contains("lex_ns") || stderr.contains("Pipeline Profile"),
        "Profiling table not found in stderr: {}",
        stderr
    );

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    cleanup();
}

#[test]
fn test_profile_json_bypasses_daemon() {
    cleanup();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success());
    thread::sleep(Duration::from_millis(300));

    // Execute with --profile-json flag (should bypass daemon)
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 * 5")
        .arg("--profile-json")
        .output()
        .expect("Failed to execute with profile-json");

    assert!(output.status.success(), "Profile-json execution failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check output is correct
    assert_eq!(stdout.trim(), "50", "Profile-json output incorrect");

    // Check JSON profiling output is present in stderr
    assert!(
        stderr.contains("lex_ns") && stderr.contains("parse_ns"),
        "JSON profiling output not found in stderr: {}",
        stderr
    );

    // Try to parse as JSON (should be valid)
    assert!(
        stderr.trim().starts_with('{') && stderr.trim().ends_with('}'),
        "Stderr should be valid JSON: {}",
        stderr
    );

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    cleanup();
}

#[test]
fn test_profiling_without_daemon_works() {
    cleanup();

    // Ensure daemon is NOT running
    assert!(!std::path::Path::new(SOCKET_PATH).exists());

    // Execute with --profile without daemon
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("100 / 4")
        .arg("--profile")
        .output()
        .expect("Failed to execute profiling without daemon");

    assert!(output.status.success(), "Profiling without daemon failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(stdout.trim(), "25", "Profiling output incorrect");
    assert!(
        stderr.contains("lex_ns") || stderr.contains("Pipeline Profile"),
        "Profiling output missing: {}",
        stderr
    );

    cleanup();
}

#[test]
fn test_profiling_and_daemon_output_match() {
    cleanup();

    let test_code = "x = 42\ny = 58\nprint(x)\nprint(y)\nx + y";

    // Execute through daemon
    let _ = Command::new(BINARY_PATH).arg("--daemon").output();

    thread::sleep(Duration::from_millis(300));

    let daemon_output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg(test_code)
        .output()
        .expect("Failed to execute through daemon");

    let daemon_result = String::from_utf8_lossy(&daemon_output.stdout).to_string();

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    thread::sleep(Duration::from_millis(200));

    // Execute with profiling (bypasses daemon even if it were running)
    let profile_output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg(test_code)
        .arg("--profile")
        .output()
        .expect("Failed to execute with profiling");

    let profile_result = String::from_utf8_lossy(&profile_output.stdout).to_string();

    // Both should produce identical output
    assert_eq!(
        daemon_result, profile_result,
        "Daemon and profiling output should match.\nDaemon: {}\nProfile: {}",
        daemon_result, profile_result
    );

    cleanup();
}

#[test]
fn test_profiling_flags_mutually_work() {
    cleanup();

    // Both --profile and --profile-json should work
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("7 * 8")
        .arg("--profile")
        .output()
        .expect("Failed with --profile");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "56");

    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("7 * 8")
        .arg("--profile-json")
        .output()
        .expect("Failed with --profile-json");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "56");

    cleanup();
}

#[test]
fn test_profiling_error_handling() {
    cleanup();

    // Execute code that causes error with --profile
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 / 0")
        .arg("--profile")
        .output()
        .expect("Failed to execute error case with profiling");

    assert!(!output.status.success(), "Division by zero should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Division by zero"),
        "Error message should mention division by zero: {}",
        stderr
    );

    cleanup();
}
