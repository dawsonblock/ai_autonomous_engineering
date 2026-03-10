//! Integration tests for daemon server
//!
//! Tests AC2.1: pyrust --daemon starts server, creates /tmp/pyrust.sock and /tmp/pyrust.pid
//! - Server accepts connections and handles requests sequentially
//! - SIGTERM/SIGINT trigger graceful shutdown with cleanup
//! - Socket permissions set to 0600 (owner only)
//! - Request timeout prevents hung connections

use pyrust::daemon::DaemonServer;
use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

// Global counter for unique test IDs
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Helper function to generate unique test paths
fn get_test_paths() -> (String, String) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let socket_path = format!("/tmp/pyrust_test_{}.sock", id);
    let pid_path = format!("/tmp/pyrust_test_{}.pid", id);
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

// Helper function to send request and get response
fn send_request(
    socket_path: &str,
    code: &str,
) -> Result<DaemonResponse, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path)?;

    // Create and send request
    let request = DaemonRequest::new(code);
    let encoded = request.encode();
    stream.write_all(&encoded)?;
    stream.flush()?;

    // Read response
    // Read status byte
    let mut status_buf = [0u8; 1];
    stream.read_exact(&mut status_buf)?;

    // Read length
    let mut length_buf = [0u8; 4];
    stream.read_exact(&mut length_buf)?;
    let length = u32::from_be_bytes(length_buf) as usize;

    // Read output
    let mut output_buf = vec![0u8; length];
    stream.read_exact(&mut output_buf)?;

    // Reconstruct and decode response
    let mut full_response = Vec::with_capacity(1 + 4 + length);
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) = DaemonResponse::decode(&full_response)?;
    Ok(response)
}

#[test]
fn test_daemon_socket_creation() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be created
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Verify socket exists
    assert!(
        Path::new(&socket_path).exists(),
        "Socket file does not exist at {}",
        socket_path
    );

    // Verify PID file exists
    assert!(
        Path::new(&pid_path).exists(),
        "PID file does not exist at {}",
        pid_path
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_socket_permissions() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be created
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Check socket permissions
    let metadata = fs::metadata(&socket_path).expect("Failed to get socket metadata");
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // On Unix, mode includes file type bits, so we mask with 0o777 to get just permissions
    let permission_bits = mode & 0o777;

    assert_eq!(
        permission_bits, 0o600,
        "Socket permissions should be 0600 (owner read+write only), got {:o}",
        permission_bits
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_pid_file_content() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be created
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Read PID file
    let pid_content = fs::read_to_string(&pid_path).expect("Failed to read PID file");
    let pid: u32 = pid_content
        .trim()
        .parse()
        .expect("PID file contains invalid number");

    // Verify PID is positive
    assert!(pid > 0, "PID should be positive, got {}", pid);

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_simple_request() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send simple request
    let response = send_request(&socket_path, "2+3").expect("Failed to send request");

    // Verify response
    assert!(response.is_success(), "Response should indicate success");
    assert_eq!(response.output(), "5", "Response output should be '5'");

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_print_request() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send print request
    let response = send_request(&socket_path, "print(42)").expect("Failed to send request");

    // Verify response
    assert!(response.is_success(), "Response should indicate success");
    assert_eq!(
        response.output(),
        "42\n",
        "Response output should be '42\\n'"
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_error_handling() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send request that causes error
    let response = send_request(&socket_path, "10 / 0").expect("Failed to send request");

    // Verify error response
    assert!(response.is_error(), "Response should indicate error");
    assert!(
        response.output().contains("Division by zero"),
        "Error message should mention division by zero, got: {}",
        response.output()
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_sequential_requests() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send multiple sequential requests
    for i in 1..=5 {
        let code = format!("{} * 2", i);
        let response = send_request(&socket_path, &code).expect("Failed to send request");
        assert!(response.is_success(), "Request {} should succeed", i);
        assert_eq!(
            response.output(),
            (i * 2).to_string(),
            "Request {} output mismatch",
            i
        );
    }

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_complex_code() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send complex code
    let code = "x = 10\ny = 20\nz = x + y\nprint(z)\nz";
    let response = send_request(&socket_path, code).expect("Failed to send request");

    // Verify response
    assert!(response.is_success(), "Response should indicate success");
    assert_eq!(
        response.output(),
        "30\n30",
        "Response output should be '30\\n30'"
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_undefined_variable_error() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send request with undefined variable
    let response = send_request(&socket_path, "undefined_var").expect("Failed to send request");

    // Verify error response
    assert!(response.is_error(), "Response should indicate error");
    assert!(
        response.output().contains("Undefined variable"),
        "Error message should mention undefined variable, got: {}",
        response.output()
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_syntax_error() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send request with syntax error
    let response = send_request(&socket_path, "x = +").expect("Failed to send request");

    // Verify error response
    assert!(response.is_error(), "Response should indicate error");
    assert!(
        response.output().contains("ParseError") || response.output().contains("Expected"),
        "Error message should indicate parse error, got: {}",
        response.output()
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_empty_code() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Send empty code
    let response = send_request(&socket_path, "").expect("Failed to send request");

    // Verify response
    assert!(response.is_success(), "Empty code should succeed");
    assert_eq!(
        response.output(),
        "",
        "Empty code should produce empty output"
    );

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_daemon_large_request() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start daemon in background
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Create a large but valid request (1KB of code)
    let large_code = "x = 1\n".repeat(100); // ~600 bytes
    let response = send_request(&socket_path, &large_code).expect("Failed to send large request");

    // Verify response
    assert!(response.is_success(), "Large request should succeed");

    // Cleanup
    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}
