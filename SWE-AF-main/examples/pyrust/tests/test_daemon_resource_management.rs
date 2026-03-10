//! Integration test for daemon resource management
//!
//! Tests resource cleanup, fork/pipe handling, and error conditions.
//! These are CRITICAL areas where bugs were fixed in iteration 2 of the merge.
//!
//! Priority: Highest (conflict resolution areas - tests fixes for resource leaks)
//!
//! Focuses on:
//! - Fork/pipe synchronization (main.rs lines 109-156)
//! - File descriptor cleanup
//! - Error handling during daemon startup
//! - Socket and PID file cleanup on shutdown

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

const BINARY_PATH: &str = "./target/release/pyrust";
const SOCKET_PATH: &str = "/tmp/pyrust.sock";
const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

fn cleanup() {
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

#[test]
fn test_daemon_startup_cleanup_on_parent_exit() {
    cleanup();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon startup failed");

    // Parent process should have exited (status.success() means parent exited with 0)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Daemon started with PID"),
        "Parent should print PID and exit: {}",
        stdout
    );

    thread::sleep(Duration::from_millis(300));

    // Verify daemon is actually running
    assert!(Path::new(SOCKET_PATH).exists(), "Socket should exist");
    assert!(Path::new(PID_FILE_PATH).exists(), "PID file should exist");

    // Verify daemon process is running
    let pid_str = fs::read_to_string(PID_FILE_PATH).expect("Failed to read PID file");
    let pid: i32 = pid_str.trim().parse().expect("Invalid PID");

    // Check if process exists
    let process_exists = unsafe { libc::kill(pid, 0) } == 0;
    assert!(process_exists, "Daemon process should be running");

    cleanup();
}

#[test]
fn test_daemon_shutdown_removes_all_resources() {
    cleanup();

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Verify resources exist
    assert!(Path::new(SOCKET_PATH).exists());
    assert!(Path::new(PID_FILE_PATH).exists());

    let pid_str = fs::read_to_string(PID_FILE_PATH).expect("Failed to read PID file");
    let pid: i32 = pid_str.trim().parse().expect("Invalid PID");

    // Stop daemon
    let output = Command::new(BINARY_PATH)
        .arg("--stop-daemon")
        .output()
        .expect("Failed to stop daemon");

    assert!(output.status.success(), "Daemon stop failed");

    thread::sleep(Duration::from_millis(200));

    // Verify all resources cleaned up
    assert!(
        !Path::new(SOCKET_PATH).exists(),
        "Socket file should be removed"
    );
    assert!(
        !Path::new(PID_FILE_PATH).exists(),
        "PID file should be removed"
    );

    // Verify process stopped
    let process_exists = unsafe { libc::kill(pid, 0) } == 0;
    assert!(!process_exists, "Daemon process should be stopped");

    cleanup();
}

#[test]
fn test_daemon_handles_sigterm_gracefully() {
    cleanup();

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    let pid_str = fs::read_to_string(PID_FILE_PATH).expect("Failed to read PID file");
    let pid: i32 = pid_str.trim().parse().expect("Invalid PID");

    // Send SIGTERM directly
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    thread::sleep(Duration::from_millis(300));

    // Verify graceful shutdown cleaned up resources
    assert!(
        !Path::new(SOCKET_PATH).exists(),
        "Socket should be removed after SIGTERM"
    );
    assert!(
        !Path::new(PID_FILE_PATH).exists(),
        "PID file should be removed after SIGTERM"
    );

    let process_exists = unsafe { libc::kill(pid, 0) } == 0;
    assert!(!process_exists, "Process should be stopped");

    cleanup();
}

#[test]
fn test_stale_socket_cleanup_on_startup() {
    cleanup();

    // Create a stale socket file (not a real socket)
    fs::write(SOCKET_PATH, "stale").expect("Failed to create stale socket");

    // Try to start daemon - should clean up stale socket and succeed
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(
        output.status.success(),
        "Daemon should clean up stale socket and start"
    );

    thread::sleep(Duration::from_millis(300));

    // Verify daemon started successfully with real socket
    assert!(Path::new(SOCKET_PATH).exists());

    // Try to connect to verify it's a real socket
    let test_output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("1 + 1")
        .output()
        .expect("Failed to execute");

    assert!(
        test_output.status.success(),
        "Should be able to execute through daemon"
    );
    assert_eq!(String::from_utf8_lossy(&test_output.stdout).trim(), "2");

    cleanup();
}

#[test]
fn test_daemon_rejects_concurrent_execution_during_startup() {
    cleanup();

    // Start two daemon processes nearly simultaneously
    let handle1 = thread::spawn(|| {
        Command::new(BINARY_PATH)
            .arg("--daemon")
            .output()
            .expect("Failed to start daemon 1")
    });

    // Small delay to ensure first one starts first
    thread::sleep(Duration::from_millis(50));

    let output2 = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon 2");

    let output1 = handle1.join().expect("Thread panicked");

    // One should succeed, one should fail
    let success_count = [output1.status.success(), output2.status.success()]
        .iter()
        .filter(|&&s| s)
        .count();

    assert_eq!(
        success_count,
        1,
        "Exactly one daemon startup should succeed.\nOutput1: {:?}\nOutput2: {:?}",
        String::from_utf8_lossy(&output1.stderr),
        String::from_utf8_lossy(&output2.stderr)
    );

    cleanup();
}

#[test]
fn test_stop_daemon_without_daemon_running() {
    cleanup();

    // Try to stop daemon when it's not running
    let output = Command::new(BINARY_PATH)
        .arg("--stop-daemon")
        .output()
        .expect("Failed to execute stop-daemon");

    assert!(
        !output.status.success(),
        "Stop-daemon should fail when daemon not running"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to stop daemon") || stderr.contains("Failed to read PID file"),
        "Error should mention failure to stop: {}",
        stderr
    );

    cleanup();
}

#[test]
fn test_daemon_execution_after_restart() {
    cleanup();

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Execute code
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("10 + 10")
        .output()
        .expect("Failed to execute");

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "20");

    // Stop daemon
    let _ = Command::new(BINARY_PATH).arg("--stop-daemon").output();

    thread::sleep(Duration::from_millis(300));

    // Start daemon again
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to restart daemon");

    assert!(output.status.success(), "Daemon restart failed");
    thread::sleep(Duration::from_millis(300));

    // Execute code through restarted daemon
    let output = Command::new(BINARY_PATH)
        .arg("-c")
        .arg("20 + 20")
        .output()
        .expect("Failed to execute after restart");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "40");

    cleanup();
}

#[test]
fn test_daemon_handles_rapid_requests() {
    cleanup();

    // Start daemon
    let _ = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    thread::sleep(Duration::from_millis(300));

    // Send 20 rapid requests
    let mut handles = vec![];

    for i in 0..20 {
        let handle = thread::spawn(move || {
            let output = Command::new(BINARY_PATH)
                .arg("-c")
                .arg(format!("{} * 2", i))
                .output()
                .expect("Failed to execute");

            (
                output.status.success(),
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                format!("{}", i * 2),
            )
        });
        handles.push(handle);
    }

    // Collect results
    let mut success_count = 0;
    for handle in handles {
        let (success, result, expected) = handle.join().expect("Thread panicked");
        if success && result == expected {
            success_count += 1;
        }
    }

    // All requests should succeed
    assert_eq!(
        success_count, 20,
        "All rapid requests should succeed, got {}/20",
        success_count
    );

    cleanup();
}
