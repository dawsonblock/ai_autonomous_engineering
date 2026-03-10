//! Integration tests for daemon client-server interaction after merge
//!
//! These tests verify the integrated behavior of daemon server and client,
//! focusing on the Cargo.toml conflict resolution where signal-hook (server)
//! and libc (client) dependencies were merged together.
//!
//! PRIORITY 1: Test conflict resolution - both dependencies work together
//! PRIORITY 2: Test cross-feature interaction - client stop triggers server signal handler
//! PRIORITY 3: Test daemon protocol integration

use pyrust::daemon::DaemonServer;
use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

// Global counter for unique test IDs
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(2000);

fn get_test_paths() -> (String, String) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    (
        format!("/tmp/pyrust_integ_{}.sock", id),
        format!("/tmp/pyrust_integ_{}.pid", id),
    )
}

fn cleanup_test_files(socket_path: &str, pid_path: &str) {
    let _ = fs::remove_file(socket_path);
    let _ = fs::remove_file(pid_path);
}

fn wait_for_socket(socket_path: &str, timeout_secs: u64) -> bool {
    use std::os::unix::net::UnixStream;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(timeout_secs) {
        if Path::new(socket_path).exists() {
            if UnixStream::connect(socket_path).is_ok() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

/// PRIORITY 1: Test that signal-hook (daemon-server) and libc (daemon-client)
/// dependencies from the Cargo.toml conflict resolution work together correctly
#[test]
fn test_conflict_resolution_signal_hook_and_libc() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon using signal-hook for signal handling
    let daemon = DaemonServer::with_paths(socket_path.clone(), pid_path.clone())
        .expect("Failed to create daemon - signal-hook dependency issue");

    let daemon_handle = thread::spawn(move || {
        let _ = daemon.run();
    });

    // Wait for daemon to initialize
    assert!(
        wait_for_socket(&socket_path, 5),
        "Daemon failed to start - possible signal-hook configuration issue"
    );

    // Verify PID file exists
    assert!(
        Path::new(&pid_path).exists(),
        "PID file not created by daemon"
    );

    // Read PID and use libc to send signal (critical integration point)
    let pid_str = fs::read_to_string(&pid_path).expect("Failed to read PID");
    let pid: i32 = pid_str.trim().parse().expect("Invalid PID");

    // Use libc::kill (daemon-client dependency) to send SIGTERM
    // This triggers signal-hook handlers in the daemon server
    #[cfg(unix)]
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    // Wait for signal-hook to process SIGTERM and cleanup
    thread::sleep(Duration::from_millis(250));

    // Verify cleanup happened via signal-hook handlers
    assert!(
        !Path::new(&socket_path).exists(),
        "signal-hook SIGTERM handler failed to remove socket"
    );
    assert!(
        !Path::new(&pid_path).exists(),
        "signal-hook SIGTERM handler failed to remove PID file"
    );

    let _ = daemon_handle.join();
    cleanup_test_files(&socket_path, &pid_path);
}

/// PRIORITY 2: Test SIGINT signal handling (also uses signal-hook)
#[test]
fn test_signal_hook_sigint_handling() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    let daemon = DaemonServer::with_paths(socket_path.clone(), pid_path.clone())
        .expect("Failed to create daemon");

    let daemon_handle = thread::spawn(move || {
        let _ = daemon.run();
    });

    assert!(wait_for_socket(&socket_path, 5), "Daemon not started");

    let pid_str = fs::read_to_string(&pid_path).expect("Failed to read PID");
    let pid: i32 = pid_str.trim().parse().expect("Invalid PID");

    // Send SIGINT using libc
    #[cfg(unix)]
    unsafe {
        libc::kill(pid, libc::SIGINT);
    }

    thread::sleep(Duration::from_millis(250));

    // Verify both signal-hook handlers work (SIGTERM and SIGINT)
    assert!(!Path::new(&socket_path).exists(), "SIGINT handler failed");
    assert!(!Path::new(&pid_path).exists(), "SIGINT handler failed");

    let _ = daemon_handle.join();
    cleanup_test_files(&socket_path, &pid_path);
}

/// PRIORITY 3: Test daemon protocol integration (request/response encoding)
#[test]
fn test_daemon_protocol_encoding_decoding() {
    // Test protocol without actual daemon to verify encoding/decoding
    let test_cases = vec![
        "2+3",
        "x = 10\ny = 20\nx + y",
        "print(42)",
        "",
        "# Comment\nx = 1",
    ];

    for code in test_cases {
        // Encode request
        let request = DaemonRequest::new(code);
        let encoded = request.encode();

        // Decode request
        let (decoded, _) = DaemonRequest::decode(&encoded).expect("Failed to decode request");
        assert_eq!(decoded.code(), code, "Request encoding mismatch");

        // Test response encoding/decoding
        let response = DaemonResponse::success("test output");
        let encoded = response.encode();
        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Failed to decode response");
        assert!(decoded.is_success());
        assert_eq!(decoded.output(), "test output");

        // Test error response
        let error_response = DaemonResponse::error("test error");
        let encoded = error_response.encode();
        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Failed to decode error");
        assert!(decoded.is_error());
        assert_eq!(decoded.output(), "test error");
    }
}

/// Test that rapid daemon start/stop cycles work with both dependencies
#[test]
fn test_rapid_start_stop_with_both_dependencies() {
    for i in 0..3 {
        let (socket_path, pid_path) = get_test_paths();
        cleanup_test_files(&socket_path, &pid_path);

        let daemon = DaemonServer::with_paths(socket_path.clone(), pid_path.clone())
            .expect(&format!("Failed on iteration {}", i));

        let daemon_handle = thread::spawn(move || {
            let _ = daemon.run();
        });

        assert!(
            wait_for_socket(&socket_path, 5),
            "Iteration {} failed to start",
            i
        );

        let pid_str = fs::read_to_string(&pid_path).unwrap();
        let pid: i32 = pid_str.trim().parse().unwrap();

        #[cfg(unix)]
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }

        thread::sleep(Duration::from_millis(250));

        assert!(
            !Path::new(&socket_path).exists(),
            "Iteration {} failed to cleanup socket",
            i
        );

        let _ = daemon_handle.join();
        cleanup_test_files(&socket_path, &pid_path);
    }
}

/// Test that daemon server can be created and configured properly
#[test]
fn test_daemon_server_initialization() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Test that daemon server can be created with custom paths
    let daemon = DaemonServer::with_paths(socket_path.clone(), pid_path.clone());
    assert!(daemon.is_ok(), "Failed to create daemon with custom paths");

    cleanup_test_files(&socket_path, &pid_path);
}

/// Test that stale socket files are handled correctly
#[test]
fn test_stale_socket_handling() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Create a stale socket file
    fs::write(&socket_path, "").expect("Failed to create stale socket");

    // Daemon should be able to start by removing stale socket
    let daemon = DaemonServer::with_paths(socket_path.clone(), pid_path.clone());
    assert!(daemon.is_ok(), "Daemon failed to handle stale socket file");

    let daemon_handle = thread::spawn(move || {
        let _ = daemon.unwrap().run();
    });

    assert!(wait_for_socket(&socket_path, 5), "Daemon not started");

    // Cleanup
    let pid_str = fs::read_to_string(&pid_path).unwrap();
    let pid: i32 = pid_str.trim().parse().unwrap();

    #[cfg(unix)]
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    thread::sleep(Duration::from_millis(250));
    let _ = daemon_handle.join();
    cleanup_test_files(&socket_path, &pid_path);
}

/// Test protocol error handling
#[test]
fn test_protocol_error_conditions() {
    // Test incomplete request
    let incomplete = vec![0, 0, 0]; // Less than 4 bytes
    let result = DaemonRequest::decode(&incomplete);
    assert!(result.is_err(), "Should reject incomplete message");

    // Test invalid UTF-8
    let mut invalid_utf8 = vec![0, 0, 0, 3]; // length = 3
    invalid_utf8.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
    let result = DaemonRequest::decode(&invalid_utf8);
    assert!(result.is_err(), "Should reject invalid UTF-8");

    // Test incomplete response
    let incomplete_response = vec![0, 0, 0, 0]; // Less than 5 bytes
    let result = DaemonResponse::decode(&incomplete_response);
    assert!(result.is_err(), "Should reject incomplete response");

    // Test invalid status code
    let mut invalid_status = vec![99]; // Invalid status
    invalid_status.extend_from_slice(&[0, 0, 0, 2]);
    invalid_status.extend_from_slice(b"ok");
    let result = DaemonResponse::decode(&invalid_status);
    assert!(result.is_err(), "Should reject invalid status code");
}
