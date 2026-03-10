//! Integration tests for daemon_client.rs conflict resolution
//!
//! Tests the interaction between:
//! - issue/04-daemon-client-redundant-closures: Direct function pointers
//! - issue/06-code-formatting: Multi-line formatting
//!
//! Priority: HIGH - This is a conflict resolution area
//!
//! Key risks:
//! - Error handling with map_err(DaemonClientError::*) direct calls
//! - Socket configuration error paths
//! - Read/Write error propagation

use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// Test socket path for isolation
const TEST_SOCKET: &str = "/tmp/pyrust_test_conflict_daemon.sock";
const TEST_PID_FILE: &str = "/tmp/pyrust_test_conflict_daemon.pid";

fn cleanup_test_files() {
    let _ = fs::remove_file(TEST_SOCKET);
    let _ = fs::remove_file(TEST_PID_FILE);
}

#[test]
fn test_daemon_client_error_mapping_connection_failed() {
    // Test that ConnectionFailed error mapping works with direct function pointer
    // This validates the closure removal didn't break error handling
    cleanup_test_files();

    // Import the client module
    use pyrust::daemon_client::DaemonClient;

    // Try to execute when daemon is not running
    // This should trigger ConnectionFailed error path with map_err
    let result = DaemonClient::execute_or_fallback("2+3");

    // Should fall back to direct execution successfully
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "5");

    cleanup_test_files();
}

#[test]
fn test_daemon_client_socket_config_error_paths() {
    // Test socket configuration error paths with direct function pointers
    // Validates that set_read_timeout and set_write_timeout map_err calls work
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    // Start a mock daemon that accepts connections but doesn't respond properly
    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        // Accept one connection but don't send proper response
        if let Ok((mut stream, _)) = listener.accept() {
            // Read the request but send malformed response
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Send incomplete header (only 3 bytes instead of 5)
            let _ = stream.write_all(&[0, 0, 0]);
        }
    });

    // Give the listener time to start
    thread::sleep(Duration::from_millis(50));

    // Execute via daemon - should handle read error gracefully
    let result = DaemonClient::execute_or_fallback("2+3");

    // Should fall back to direct execution
    assert!(result.is_ok());

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_client_write_error_propagation() {
    // Test that WriteFailed error is properly mapped with direct function pointer
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    // Create socket but close it immediately
    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        // Accept connection and immediately drop it
        if let Ok((stream, _)) = listener.accept() {
            drop(stream);
        }
    });

    thread::sleep(Duration::from_millis(50));

    // Try to execute - write should fail
    let result = DaemonClient::execute_or_fallback("print('test')");

    // Should fall back to direct execution
    assert!(result.is_ok());

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_client_read_error_propagation() {
    // Test that ReadFailed error is properly mapped with direct function pointer
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    // Start daemon that sends incomplete response
    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Send only part of header then close
            let _ = stream.write_all(&[1]); // Only 1 byte of 5-byte header
            drop(stream);
        }
    });

    thread::sleep(Duration::from_millis(50));

    let result = DaemonClient::execute_or_fallback("42");

    // Should fall back to direct execution
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_client_protocol_error_propagation() {
    // Test that protocol errors are properly mapped
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Send invalid status code and malformed response
            let invalid_response = [99u8, 0, 0, 0, 1, 65]; // Invalid status + 1 byte payload
            let _ = stream.write_all(&invalid_response);
        }
    });

    thread::sleep(Duration::from_millis(50));

    let result = DaemonClient::execute_or_fallback("1+1");

    // Should fall back to direct execution on protocol error
    assert!(result.is_ok());

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_client_response_too_large_protection() {
    // Test that response size limit protection works
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Send header claiming 20MB response (exceeds 10MB limit)
            let size_bytes: [u8; 4] = 20_000_000u32.to_be_bytes();
            let header = [
                1u8,
                size_bytes[0],
                size_bytes[1],
                size_bytes[2],
                size_bytes[3],
            ];
            let _ = stream.write_all(&header);
            // Don't send actual body - client should reject based on size alone
        }
    });

    thread::sleep(Duration::from_millis(50));

    let result = DaemonClient::execute_or_fallback("'x' * 1000");

    // Should fall back to direct execution when response is too large
    assert!(result.is_ok());

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_client_successful_execution_path() {
    // Test that successful path still works after closure removal and formatting
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;
    use pyrust::daemon_protocol::DaemonResponse;

    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            // Read request
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).unwrap();
            let code_len = u32::from_be_bytes(len_buf) as usize;
            let mut code_buf = vec![0u8; code_len];
            stream.read_exact(&mut code_buf).unwrap();

            // Send success response
            let response = DaemonResponse::success("5");
            let response_bytes = response.encode();
            stream.write_all(&response_bytes).unwrap();
        }
    });

    thread::sleep(Duration::from_millis(50));

    let result = DaemonClient::execute_or_fallback("2+3");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "5");

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_client_execution_error_propagation() {
    // Test that execution errors from daemon are properly handled
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;
    use pyrust::daemon_protocol::DaemonResponse;

    let listener = UnixListener::bind(TEST_SOCKET).expect("Failed to bind test socket");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            // Read and ignore request
            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).unwrap();
            let code_len = u32::from_be_bytes(len_buf) as usize;
            let mut code_buf = vec![0u8; code_len];
            stream.read_exact(&mut code_buf).unwrap();

            // Send error response
            let response = DaemonResponse::error("Division by zero");
            let response_bytes = response.encode();
            stream.write_all(&response_bytes).unwrap();
        }
    });

    thread::sleep(Duration::from_millis(50));

    // This should fall back when daemon returns error
    let result = DaemonClient::execute_or_fallback("1/0");

    // Fallback to direct execution which will also error
    assert!(result.is_err());

    handle.join().unwrap();
    cleanup_test_files();
}

#[test]
fn test_daemon_status_formatting() {
    // Test daemon status string formatting (formatting merge verification)
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    // When daemon is not running
    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is not running");

    // Create socket to simulate running daemon
    fs::write(TEST_SOCKET, "").unwrap();

    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is running");

    cleanup_test_files();
}

#[test]
fn test_is_daemon_running_edge_cases() {
    // Test daemon detection edge cases
    cleanup_test_files();

    use pyrust::daemon_client::DaemonClient;

    // Socket doesn't exist
    assert!(!DaemonClient::is_daemon_running());

    // Socket exists as directory (edge case)
    if Path::new(TEST_SOCKET).exists() {
        fs::remove_file(TEST_SOCKET).unwrap();
    }

    // Create socket file
    fs::write(TEST_SOCKET, "test").unwrap();
    assert!(DaemonClient::is_daemon_running());

    cleanup_test_files();
}
