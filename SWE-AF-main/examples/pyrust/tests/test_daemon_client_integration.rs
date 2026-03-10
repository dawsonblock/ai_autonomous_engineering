//! Integration tests for daemon client with actual daemon lifecycle
//!
//! Testing strategy from issue:
//! - Start daemon, call execute_or_fallback('2+3'), verify correct result
//! - Stop daemon, call again, verify fallback works
//! - Test error propagation with division by zero

use pyrust::daemon::DaemonServer;
use pyrust::daemon_client::DaemonClient;
use std::fs;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use std::thread;
use std::time::Duration;

// Mutex to serialize tests that use the default daemon socket
static DAEMON_TEST_LOCK: Mutex<()> = Mutex::new(());

// Global counter for unique test IDs
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Helper function to generate unique test paths
fn get_test_paths() -> (String, String) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let socket_path = format!("/tmp/pyrust_integration_test_{}.sock", id);
    let pid_path = format!("/tmp/pyrust_integration_test_{}.pid", id);
    (socket_path, pid_path)
}

// Helper function to clean up test files
fn cleanup_test_files(socket_path: &str, pid_path: &str) {
    let _ = fs::remove_file(socket_path);
    let _ = fs::remove_file(pid_path);
}

// Helper function to start daemon in background thread
fn start_daemon_in_background(socket_path: String, pid_path: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let daemon =
            DaemonServer::with_paths(socket_path, pid_path).expect("Failed to create daemon");
        let _ = daemon.run();
    })
}

// Helper function to wait for socket to be available
fn wait_for_socket(socket_path: &str, timeout_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(timeout_secs) {
        if Path::new(socket_path).exists() {
            // Try to connect to ensure it's ready
            if UnixStream::connect(socket_path).is_ok() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

// Helper function to stop daemon via PID file
fn stop_daemon_via_pid(pid_path: &str) -> Result<(), String> {
    let pid_str =
        fs::read_to_string(pid_path).map_err(|e| format!("Failed to read PID file: {}", e))?;

    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|e| format!("Invalid PID: {}", e))?;

    // Send SIGTERM
    #[cfg(unix)]
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    // Wait for cleanup
    thread::sleep(Duration::from_millis(200));

    Ok(())
}

/// Integration test: Start daemon, execute via daemon, stop daemon, verify fallback
#[test]
fn test_daemon_client_full_lifecycle() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Temporarily override the socket path for DaemonClient
    // Since DaemonClient uses hardcoded paths, we need to use the default path
    // Clean up default paths
    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Start daemon with default paths
    let _handle = start_daemon_in_background(
        "/tmp/pyrust.sock".to_string(),
        "/tmp/pyrust.pid".to_string(),
    );

    // Wait for daemon to be ready
    assert!(
        wait_for_socket("/tmp/pyrust.sock", 5),
        "Daemon did not start within 5 seconds"
    );

    // Test 1: Execute via daemon (should succeed)
    let result = DaemonClient::execute_or_fallback("2+3").expect("Failed to execute via daemon");
    assert_eq!(result, "5", "Daemon execution should return '5'");

    // Test 2: Execute more complex code via daemon
    let result = DaemonClient::execute_or_fallback("x = 10\ny = 20\nx + y")
        .expect("Failed to execute complex code via daemon");
    assert_eq!(result, "30", "Daemon execution should return '30'");

    // Test 3: Error propagation through daemon
    let result = DaemonClient::execute_or_fallback("10 / 0");
    assert!(result.is_err(), "Division by zero should return error");
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("Division by zero"),
        "Error should contain 'Division by zero', got: {}",
        error_msg
    );

    // Stop daemon
    stop_daemon_via_pid("/tmp/pyrust.pid").expect("Failed to stop daemon");

    // Verify socket is removed
    thread::sleep(Duration::from_millis(300));
    assert!(
        !Path::new("/tmp/pyrust.sock").exists(),
        "Socket should be removed after daemon shutdown"
    );

    // Test 4: Fallback execution after daemon stopped
    let result =
        DaemonClient::execute_or_fallback("2+3").expect("Fallback execution should succeed");
    assert_eq!(result, "5", "Fallback execution should return '5'");

    // Test 5: Fallback with complex code
    let result = DaemonClient::execute_or_fallback("print(42)\n50")
        .expect("Fallback with print should succeed");
    assert_eq!(result, "42\n50", "Fallback should handle print correctly");

    // Test 6: Error propagation in fallback mode
    let result = DaemonClient::execute_or_fallback("undefined_variable");
    assert!(result.is_err(), "Undefined variable should return error");
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("Undefined variable"),
        "Error should contain 'Undefined variable', got: {}",
        error_msg
    );

    // Cleanup
    cleanup_test_files("/tmp/pyrust.sock", "/tmp/pyrust.pid");
}

/// Test connection timeout prevents hung requests
#[test]
fn test_daemon_client_connection_timeout() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    // Ensure no daemon is running
    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Try to execute - should fail quickly and fallback
    let start = std::time::Instant::now();
    let result =
        DaemonClient::execute_or_fallback("2+3").expect("Should fallback to direct execution");
    let duration = start.elapsed();

    // Should complete quickly via fallback (< 1 second)
    assert!(
        duration < Duration::from_secs(1),
        "Connection timeout and fallback should be fast, took {:?}",
        duration
    );

    assert_eq!(result, "5", "Fallback should return correct result");
}

/// Test is_daemon_running detection
#[test]
fn test_daemon_client_is_daemon_running() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Initially not running
    assert!(
        !DaemonClient::is_daemon_running(),
        "Daemon should not be running"
    );

    // Start daemon
    let _handle = start_daemon_in_background(
        "/tmp/pyrust.sock".to_string(),
        "/tmp/pyrust.pid".to_string(),
    );

    // Wait for daemon
    wait_for_socket("/tmp/pyrust.sock", 5);

    // Should detect running daemon
    assert!(
        DaemonClient::is_daemon_running(),
        "Daemon should be detected as running"
    );

    // Stop daemon
    stop_daemon_via_pid("/tmp/pyrust.pid").ok();
    thread::sleep(Duration::from_millis(300));

    // Should detect stopped daemon
    assert!(
        !DaemonClient::is_daemon_running(),
        "Daemon should be detected as stopped"
    );

    // Cleanup
    cleanup_test_files("/tmp/pyrust.sock", "/tmp/pyrust.pid");
}

/// Test daemon_status returns correct status string
#[test]
fn test_daemon_client_status_string() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Initially not running
    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is not running");

    // Start daemon
    let _handle = start_daemon_in_background(
        "/tmp/pyrust.sock".to_string(),
        "/tmp/pyrust.pid".to_string(),
    );

    // Wait for daemon
    wait_for_socket("/tmp/pyrust.sock", 5);

    // Should show running
    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is running");

    // Stop daemon
    stop_daemon_via_pid("/tmp/pyrust.pid").ok();
    thread::sleep(Duration::from_millis(300));

    // Should show not running
    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is not running");

    // Cleanup
    cleanup_test_files("/tmp/pyrust.sock", "/tmp/pyrust.pid");
}

/// Test stop_daemon command
#[test]
fn test_daemon_client_stop_daemon_command() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Start daemon
    let _handle = start_daemon_in_background(
        "/tmp/pyrust.sock".to_string(),
        "/tmp/pyrust.pid".to_string(),
    );

    // Wait for daemon
    wait_for_socket("/tmp/pyrust.sock", 5);

    // Verify daemon is running
    assert!(DaemonClient::is_daemon_running());

    // Stop daemon using DaemonClient
    DaemonClient::stop_daemon().expect("stop_daemon should succeed");

    // Verify daemon stopped
    assert!(
        !DaemonClient::is_daemon_running(),
        "Daemon should be stopped"
    );
    assert!(
        !Path::new("/tmp/pyrust.sock").exists(),
        "Socket should be removed"
    );

    // Cleanup
    cleanup_test_files("/tmp/pyrust.sock", "/tmp/pyrust.pid");
}

/// Test error propagation matches direct execution format
#[test]
fn test_daemon_client_error_format_consistency() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Start daemon
    let _handle = start_daemon_in_background(
        "/tmp/pyrust.sock".to_string(),
        "/tmp/pyrust.pid".to_string(),
    );

    // Wait for daemon
    wait_for_socket("/tmp/pyrust.sock", 5);

    // Test error via daemon
    let daemon_error = DaemonClient::execute_or_fallback("10 / 0");
    assert!(daemon_error.is_err());
    let daemon_msg = format!("{}", daemon_error.unwrap_err());

    // Stop daemon
    stop_daemon_via_pid("/tmp/pyrust.pid").ok();
    thread::sleep(Duration::from_millis(300));

    // Test error via fallback
    let fallback_error = DaemonClient::execute_or_fallback("10 / 0");
    assert!(fallback_error.is_err());
    let fallback_msg = format!("{}", fallback_error.unwrap_err());

    // Both should mention division by zero
    assert!(
        daemon_msg.contains("Division by zero"),
        "Daemon error: {}",
        daemon_msg
    );
    assert!(
        fallback_msg.contains("Division by zero"),
        "Fallback error: {}",
        fallback_msg
    );

    // Cleanup
    cleanup_test_files("/tmp/pyrust.sock", "/tmp/pyrust.pid");
}

/// Test multiple sequential requests with fallback
#[test]
fn test_daemon_client_sequential_requests_with_fallback() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Execute multiple requests without daemon (fallback mode)
    for i in 1..=10 {
        let code = format!("{} * 2", i);
        let result =
            DaemonClient::execute_or_fallback(&code).expect("Fallback execution should succeed");
        assert_eq!(result, (i * 2).to_string());
    }
}

/// Test that socket existence check is reliable
#[test]
fn test_daemon_client_socket_detection() {
    let _lock = DAEMON_TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");

    // No socket - daemon not running
    assert!(!DaemonClient::is_daemon_running());

    // Create fake socket file (not actually listening)
    fs::write("/tmp/pyrust.sock", "").ok();

    // Should detect socket exists
    assert!(DaemonClient::is_daemon_running());

    // Cleanup
    fs::remove_file("/tmp/pyrust.sock").ok();
}
