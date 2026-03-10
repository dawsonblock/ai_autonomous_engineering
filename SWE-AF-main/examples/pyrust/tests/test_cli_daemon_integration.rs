//! Integration tests for CLI-daemon interaction after merge
//!
//! Tests the integration between main.rs CLI and daemon functionality,
//! focusing on command-line flag interactions and execution path routing.
//!
//! PRIORITY 1: CLI flags correctly route to daemon vs direct execution
//! PRIORITY 2: Profiling mode bypasses daemon (always direct execution)
//! PRIORITY 3: Cache integration works correctly in daemon mode

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const BINARY_PATH: &str = "./target/release/pyrust";
const SOCKET_PATH: &str = "/tmp/pyrust.sock";
const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

/// Helper to cleanup daemon artifacts
fn cleanup_daemon() {
    // Stop daemon if running
    if Path::new(PID_FILE_PATH).exists() {
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

/// Helper to wait for socket to appear
fn wait_for_socket(timeout_ms: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < timeout_ms as u128 {
        if Path::new(SOCKET_PATH).exists() {
            // Also try to connect to verify daemon is listening
            if std::os::unix::net::UnixStream::connect(SOCKET_PATH).is_ok() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

/// PRIORITY 1: Test that CLI routes -c code execution through daemon when available
#[test]
fn test_cli_routes_to_daemon_when_available() {
    cleanup_daemon();

    // Start daemon via CLI
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute code via CLI - should route through daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "2+3"])
        .output()
        .expect("Failed to execute code");

    assert!(output.status.success(), "Execution failed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "5");

    // Verify daemon is still running (wasn't shut down by execution)
    assert!(Path::new(SOCKET_PATH).exists(), "Daemon socket disappeared");
    assert!(
        Path::new(PID_FILE_PATH).exists(),
        "Daemon PID file disappeared"
    );

    cleanup_daemon();
}

/// PRIORITY 1: Test that CLI falls back to direct execution when daemon not available
#[test]
fn test_cli_fallback_when_daemon_unavailable() {
    cleanup_daemon();

    // Verify daemon is NOT running
    assert!(!Path::new(SOCKET_PATH).exists(), "Socket should not exist");

    // Execute code via CLI - should fallback to direct execution
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "10*5"])
        .output()
        .expect("Failed to execute code");

    assert!(output.status.success(), "Execution failed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "50");

    // Verify no daemon was started (fallback is truly direct execution)
    assert!(
        !Path::new(SOCKET_PATH).exists(),
        "Daemon should not have started"
    );

    cleanup_daemon();
}

/// PRIORITY 2: Test that --profile flag ALWAYS uses direct execution (bypasses daemon)
#[test]
fn test_profile_flag_bypasses_daemon() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute with --profile flag - should bypass daemon and use direct execution
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "2+3", "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute code");

    assert!(output.status.success(), "Execution with --profile failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify output is correct
    assert_eq!(stdout.trim(), "5");

    // Verify profiling table appears in stderr
    assert!(
        stderr.contains("Stage Breakdown"),
        "Profiling output not found"
    );
    assert!(stderr.contains("Lex"), "Profiling stages missing");
    assert!(stderr.contains("Parse"), "Profiling stages missing");
    assert!(stderr.contains("Compile"), "Profiling stages missing");
    assert!(stderr.contains("VM Execute"), "Profiling stages missing");

    cleanup_daemon();
}

/// PRIORITY 2: Test that --profile-json flag ALWAYS uses direct execution (bypasses daemon)
#[test]
fn test_profile_json_flag_bypasses_daemon() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute with --profile-json flag - should bypass daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "2+3", "--profile-json"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute code");

    assert!(
        output.status.success(),
        "Execution with --profile-json failed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify output is correct
    assert_eq!(stdout.trim(), "5");

    // Verify JSON profiling output in stderr
    assert!(
        stderr.contains("\"lex_ns\""),
        "JSON profiling output not found"
    );
    assert!(
        stderr.contains("\"parse_ns\""),
        "JSON profiling output not found"
    );
    assert!(
        stderr.contains("\"compile_ns\""),
        "JSON profiling output not found"
    );
    assert!(
        stderr.contains("\"vm_execute_ns\""),
        "JSON profiling output not found"
    );
    assert!(
        stderr.contains("\"total_ns\""),
        "JSON profiling output not found"
    );

    cleanup_daemon();
}

/// PRIORITY 1: Test CLI --stop-daemon command correctly stops the daemon
#[test]
fn test_cli_stop_daemon_command() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Get PID before stopping
    let pid_str = fs::read_to_string(PID_FILE_PATH).expect("Failed to read PID");
    let pid: i32 = pid_str.trim().parse().expect("Invalid PID");

    // Stop daemon via CLI
    let output = Command::new(BINARY_PATH)
        .arg("--stop-daemon")
        .output()
        .expect("Failed to stop daemon");

    assert!(output.status.success(), "Daemon stop failed");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("stopped"),
        "Expected stop message"
    );

    // Wait for cleanup
    thread::sleep(Duration::from_millis(300));

    // Verify cleanup happened
    assert!(!Path::new(SOCKET_PATH).exists(), "Socket not removed");
    assert!(!Path::new(PID_FILE_PATH).exists(), "PID file not removed");

    // Verify process actually stopped
    let is_running = unsafe { libc::kill(pid, 0) == 0 };
    assert!(!is_running, "Daemon process still running");

    cleanup_daemon();
}

/// PRIORITY 1: Test CLI --daemon-status command reports correctly
#[test]
fn test_cli_daemon_status_command() {
    cleanup_daemon();

    // Test status when daemon NOT running
    let output = Command::new(BINARY_PATH)
        .arg("--daemon-status")
        .output()
        .expect("Failed to check status");

    assert!(
        !output.status.success(),
        "Status should exit 1 when daemon not running"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("not running"),
        "Expected 'not running' status"
    );

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Test status when daemon IS running
    let output = Command::new(BINARY_PATH)
        .arg("--daemon-status")
        .output()
        .expect("Failed to check status");

    assert!(
        output.status.success(),
        "Status should exit 0 when daemon running"
    );
    let status_msg = String::from_utf8_lossy(&output.stdout);
    assert!(status_msg.contains("running"), "Expected 'running' status");

    cleanup_daemon();
}

/// PRIORITY 3: Test that cache works correctly with daemon execution
#[test]
fn test_cache_integration_with_daemon() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute same code multiple times - should use cache after first execution
    let code = "x = 10\ny = 20\nx + y";

    for i in 0..5 {
        let output = Command::new(BINARY_PATH)
            .args(&["-c", code])
            .output()
            .expect("Failed to execute code");

        assert!(output.status.success(), "Execution {} failed", i);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "30",
            "Incorrect result on execution {}",
            i
        );
    }

    // Execute different code to verify cache doesn't confuse different inputs
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "5 * 10"])
        .output()
        .expect("Failed to execute code");

    assert!(output.status.success(), "Different code execution failed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "50");

    cleanup_daemon();
}

/// PRIORITY 1: Test error handling through CLI-daemon integration
#[test]
fn test_cli_error_propagation_through_daemon() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Test division by zero error through CLI-daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "10 / 0"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute code");

    assert!(!output.status.success(), "Should fail with error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Division by zero"),
        "Expected division by zero error"
    );

    // Test undefined variable error through CLI-daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "undefined_var"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute code");

    assert!(!output.status.success(), "Should fail with error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Undefined variable"),
        "Expected undefined variable error"
    );

    cleanup_daemon();
}

/// PRIORITY 2: Test CLI handles both profiling flags correctly
#[test]
fn test_cli_profiling_flags_mutually_work() {
    cleanup_daemon();

    // Test --profile flag without daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "2+3", "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute with --profile");

    assert!(output.status.success(), "--profile execution failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Stage Breakdown"),
        "--profile output missing"
    );

    // Test --profile-json flag without daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "2+3", "--profile-json"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute with --profile-json");

    assert!(output.status.success(), "--profile-json execution failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("\"total_ns\""),
        "--profile-json output missing"
    );

    cleanup_daemon();
}

/// PRIORITY 1: Test file execution mode works with daemon
#[test]
fn test_cli_file_mode_with_daemon() {
    cleanup_daemon();

    // Create a test file
    let test_file = "/tmp/pyrust_test_script.py";
    fs::write(test_file, "x = 100\ny = 50\nprint(x + y)\nx - y")
        .expect("Failed to write test file");

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute file via CLI (should route through daemon)
    let output = Command::new(BINARY_PATH)
        .arg(test_file)
        .output()
        .expect("Failed to execute file");

    assert!(output.status.success(), "File execution failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should print "150\n" and then result "50" (no newline)
    assert!(stdout.contains("150"), "Print output missing");
    assert!(stdout.ends_with("50"), "Expression result missing");

    // Cleanup
    fs::remove_file(test_file).expect("Failed to remove test file");
    cleanup_daemon();
}
