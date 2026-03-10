//! Integration test for daemon-CLI-cache interaction
//!
//! Tests the interaction between:
//! - CLI commands (main.rs)
//! - Daemon client/server (daemon_client.rs, daemon.rs)
//! - Global cache (lib.rs GLOBAL_CACHE)
//!
//! Priority: High (tests cross-feature interaction)

use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

const BINARY_PATH: &str = "./target/release/pyrust";
const SOCKET_PATH: &str = "/tmp/pyrust.sock";
const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

/// Cleanup helper to ensure clean test state
fn cleanup() {
    // Stop daemon if running
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

    // Remove files
    let _ = fs::remove_file(SOCKET_PATH);
    let _ = fs::remove_file(PID_FILE_PATH);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_uses_global_cache_across_requests() {
    cleanup();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon startup failed");
    thread::sleep(Duration::from_millis(300));

    // Verify daemon started
    assert!(
        std::path::Path::new(SOCKET_PATH).exists(),
        "Socket not created"
    );

    // Execute same code multiple times through daemon
    // This should benefit from global cache
    let test_code = "x = 100\ny = 200\nx + y";

    for i in 0..5 {
        let output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(test_code)
            .output()
            .expect("Failed to execute through daemon");

        assert!(output.status.success(), "Execution {} failed", i);
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(result, "300", "Execution {} returned incorrect result", i);
    }

    // Stop daemon
    let output = Command::new(BINARY_PATH)
        .arg("--stop-daemon")
        .output()
        .expect("Failed to stop daemon");

    assert!(output.status.success(), "Daemon stop failed");

    cleanup();
}

#[test]
fn test_fallback_uses_thread_local_cache() {
    cleanup();

    // Ensure daemon is NOT running
    assert!(!std::path::Path::new(SOCKET_PATH).exists());

    // Execute same code multiple times without daemon
    // This should use thread-local cache (fallback mode)
    let test_code = "a = 50\nb = 75\na * b";

    for i in 0..5 {
        let output = Command::new(BINARY_PATH)
            .arg("-c")
            .arg(test_code)
            .output()
            .expect("Failed to execute in fallback mode");

        assert!(output.status.success(), "Fallback execution {} failed", i);
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(
            result, "3750",
            "Fallback execution {} returned incorrect result",
            i
        );
    }

    cleanup();
}

#[test]
fn test_clear_cache_command_integration() {
    cleanup();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success());
    thread::sleep(Duration::from_millis(300));

    // Execute code to populate cache
    let _ = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 + 20")
        .output()
        .expect("Failed to execute");

    // Clear cache
    let output = Command::new(BINARY_PATH)
        .arg("--clear-cache")
        .output()
        .expect("Failed to clear cache");

    assert!(output.status.success(), "Clear cache command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Cache cleared successfully"),
        "Unexpected clear cache output: {}",
        stdout
    );

    // Execute code again (should compile again, not use cache)
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 + 20")
        .output()
        .expect("Failed to execute after cache clear");

    assert!(output.status.success());
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(result, "30");

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    cleanup();
}

#[test]
fn test_daemon_status_command_accuracy() {
    cleanup();

    // Check status when daemon not running
    let output = Command::new(BINARY_PATH)
        .arg("--daemon-status")
        .output()
        .expect("Failed to check daemon status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Daemon is not running"),
        "Status should show not running: {}",
        stdout
    );
    assert!(
        !output.status.success(),
        "Exit code should be 1 when not running"
    );

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success());
    thread::sleep(Duration::from_millis(300));

    // Check status when daemon IS running
    let output = Command::new(BINARY_PATH)
        .arg("--daemon-status")
        .output()
        .expect("Failed to check daemon status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Daemon is running"),
        "Status should show running: {}",
        stdout
    );
    assert!(
        output.status.success(),
        "Exit code should be 0 when running"
    );

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    cleanup();
}

#[test]
fn test_daemon_prevents_double_start() {
    cleanup();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success());
    thread::sleep(Duration::from_millis(300));

    // Try to start daemon again
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to execute second daemon start");

    assert!(!output.status.success(), "Second daemon start should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already running"),
        "Error should mention daemon already running: {}",
        stderr
    );

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    cleanup();
}

#[test]
fn test_cache_isolation_between_daemon_and_fallback() {
    cleanup();

    // Execute in fallback mode (thread-local cache)
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("fallback = 123\nfallback")
        .output()
        .expect("Failed to execute in fallback");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "123");

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success());
    thread::sleep(Duration::from_millis(300));

    // Execute through daemon (global cache)
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("daemon = 456\ndaemon")
        .output()
        .expect("Failed to execute through daemon");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "456");

    // Stop daemon and execute in fallback again
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    thread::sleep(Duration::from_millis(200));

    // Execute in fallback mode again
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("final = 789\nfinal")
        .output()
        .expect("Failed to execute in fallback after daemon stop");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "789");

    cleanup();
}
