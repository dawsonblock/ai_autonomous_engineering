//! Integration QA: Daemon Client-Server Full Flow
//!
//! PRIORITY 2: Tests cross-feature interactions between merged branches
//!
//! Merged features:
//! - issue/13-daemon-server: Server accepts connections, handles requests
//! - issue/14-daemon-client: Client connects to server and executes code
//!
//! This test verifies end-to-end flow from client to server and back,
//! ensuring the integration between the two features works correctly.

use pyrust::daemon::DaemonServer;
use pyrust::daemon_client::DaemonClient;
use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

// Mutex to serialize tests using default daemon paths
static TEST_LOCK: Mutex<()> = Mutex::new(());
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(5000);

fn get_test_paths() -> (String, String) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    (
        format!("/tmp/pyrust_qa_flow_test_{}.sock", id),
        format!("/tmp/pyrust_qa_flow_test_{}.pid", id),
    )
}

fn cleanup_test_files(socket_path: &str, pid_path: &str) {
    let _ = fs::remove_file(socket_path);
    let _ = fs::remove_file(pid_path);
}

fn start_daemon_in_background(socket_path: String, pid_path: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let daemon =
            DaemonServer::with_paths(socket_path, pid_path).expect("Failed to create daemon");
        let _ = daemon.run();
    })
}

fn wait_for_socket(socket_path: &str, timeout_secs: u64) -> bool {
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

fn stop_daemon_via_pid(pid_path: &str) {
    if let Ok(pid_str) = fs::read_to_string(pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            #[cfg(unix)]
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
            thread::sleep(Duration::from_millis(200));
        }
    }
}

/// Test full flow: Client request → Protocol encoding → Server processing → Response
#[test]
fn test_client_server_simple_arithmetic_flow() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Start server
    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());
    assert!(wait_for_socket(&socket_path, 5), "Server should start");

    // Client creates request
    let request = DaemonRequest::new("10 + 20");
    let encoded_request = request.encode();

    // Client sends request via Unix socket
    let mut stream = UnixStream::connect(&socket_path).expect("Client should connect to server");
    stream
        .write_all(&encoded_request)
        .expect("Client should send request");
    stream.flush().expect("Client should flush");

    // Client reads response
    let mut status_buf = [0u8; 1];
    stream
        .read_exact(&mut status_buf)
        .expect("Should read status");

    let mut length_buf = [0u8; 4];
    stream
        .read_exact(&mut length_buf)
        .expect("Should read length");
    let length = u32::from_be_bytes(length_buf) as usize;

    let mut output_buf = vec![0u8; length];
    stream
        .read_exact(&mut output_buf)
        .expect("Should read output");

    // Reconstruct and decode response
    let mut full_response = Vec::new();
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) = DaemonResponse::decode(&full_response).expect("Should decode response");

    // Verify result
    assert!(response.is_success(), "Response should be success");
    assert_eq!(response.output(), "30", "Result should be 30");

    cleanup_test_files(&socket_path, &pid_path);
}

/// Test client-server interaction with error propagation
#[test]
fn test_client_server_error_propagation_flow() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());
    assert!(wait_for_socket(&socket_path, 5), "Server should start");

    // Send error-inducing request
    let request = DaemonRequest::new("100 / 0");
    let encoded_request = request.encode();

    let mut stream = UnixStream::connect(&socket_path).expect("Should connect");
    stream.write_all(&encoded_request).expect("Should send");
    stream.flush().expect("Should flush");

    // Read response
    let mut status_buf = [0u8; 1];
    stream
        .read_exact(&mut status_buf)
        .expect("Should read status");

    let mut length_buf = [0u8; 4];
    stream
        .read_exact(&mut length_buf)
        .expect("Should read length");
    let length = u32::from_be_bytes(length_buf) as usize;

    let mut output_buf = vec![0u8; length];
    stream
        .read_exact(&mut output_buf)
        .expect("Should read output");

    let mut full_response = Vec::new();
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) =
        DaemonResponse::decode(&full_response).expect("Should decode error response");

    // Verify error propagation
    assert!(response.is_error(), "Response should indicate error");
    assert!(
        response.output().contains("Division by zero"),
        "Error message should be preserved"
    );

    cleanup_test_files(&socket_path, &pid_path);
}

/// Test client-server with complex code involving variables and expressions
#[test]
fn test_client_server_complex_code_flow() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());
    assert!(wait_for_socket(&socket_path, 5), "Server should start");

    let complex_code = "a = 5\nb = 10\nc = 15\nresult = a + b * c\nresult";
    let request = DaemonRequest::new(complex_code);
    let encoded_request = request.encode();

    let mut stream = UnixStream::connect(&socket_path).expect("Should connect");
    stream.write_all(&encoded_request).expect("Should send");
    stream.flush().expect("Should flush");

    // Read response
    let mut status_buf = [0u8; 1];
    stream
        .read_exact(&mut status_buf)
        .expect("Should read status");

    let mut length_buf = [0u8; 4];
    stream
        .read_exact(&mut length_buf)
        .expect("Should read length");
    let length = u32::from_be_bytes(length_buf) as usize;

    let mut output_buf = vec![0u8; length];
    stream
        .read_exact(&mut output_buf)
        .expect("Should read output");

    let mut full_response = Vec::new();
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) = DaemonResponse::decode(&full_response).expect("Should decode response");

    assert!(response.is_success(), "Complex code should succeed");
    assert_eq!(response.output(), "155", "Result should be 5 + 10*15 = 155");

    cleanup_test_files(&socket_path, &pid_path);
}

/// Test multiple sequential client requests to same server
#[test]
fn test_client_server_sequential_requests_flow() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());
    assert!(wait_for_socket(&socket_path, 5), "Server should start");

    let test_cases = vec![
        ("2+2", "4"),
        ("10*10", "100"),
        ("50-25", "25"),
        ("100 // 3", "33"),
        ("17 % 5", "2"),
    ];

    for (code, expected) in test_cases {
        let request = DaemonRequest::new(code);
        let encoded = request.encode();

        let mut stream = UnixStream::connect(&socket_path).expect("Should connect");
        stream.write_all(&encoded).expect("Should send");
        stream.flush().expect("Should flush");

        // Read response
        let mut status_buf = [0u8; 1];
        stream
            .read_exact(&mut status_buf)
            .expect("Should read status");

        let mut length_buf = [0u8; 4];
        stream
            .read_exact(&mut length_buf)
            .expect("Should read length");
        let length = u32::from_be_bytes(length_buf) as usize;

        let mut output_buf = vec![0u8; length];
        stream
            .read_exact(&mut output_buf)
            .expect("Should read output");

        let mut full_response = Vec::new();
        full_response.extend_from_slice(&status_buf);
        full_response.extend_from_slice(&length_buf);
        full_response.extend_from_slice(&output_buf);

        let (response, _) = DaemonResponse::decode(&full_response).expect("Should decode");

        assert!(response.is_success(), "Request '{}' should succeed", code);
        assert_eq!(
            response.output(),
            expected,
            "Request '{}' output mismatch",
            code
        );
    }

    cleanup_test_files(&socket_path, &pid_path);
}

/// Test DaemonClient execute_or_fallback with running server
#[test]
fn test_daemon_client_execute_with_server_running() {
    let _lock = TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Start server on default paths
    let _handle = start_daemon_in_background(
        "/tmp/pyrust.sock".to_string(),
        "/tmp/pyrust.pid".to_string(),
    );
    assert!(
        wait_for_socket("/tmp/pyrust.sock", 5),
        "Server should start"
    );

    // Use DaemonClient which should use the running server
    let result = DaemonClient::execute_or_fallback("15 + 25").expect("Should execute via daemon");
    assert_eq!(result, "40", "Should return correct result");

    // Stop daemon
    stop_daemon_via_pid("/tmp/pyrust.pid");
    thread::sleep(Duration::from_millis(300));

    cleanup_test_files("/tmp/pyrust.sock", "/tmp/pyrust.pid");
}

/// Test DaemonClient fallback when server not running
#[test]
fn test_daemon_client_fallback_when_server_down() {
    let _lock = TEST_LOCK.lock().unwrap();

    let _ = fs::remove_file("/tmp/pyrust.sock");
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // No server running - should fallback to direct execution
    let result =
        DaemonClient::execute_or_fallback("7 * 8").expect("Should fallback to direct execution");
    assert_eq!(result, "56", "Fallback should return correct result");
}

/// Test that server continues working after client disconnects abruptly
#[test]
fn test_server_resilience_after_client_disconnect() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());
    assert!(wait_for_socket(&socket_path, 5), "Server should start");

    // Connect and disconnect without sending anything
    {
        let _stream = UnixStream::connect(&socket_path).expect("Should connect");
        // Drop stream immediately (simulates abrupt disconnect)
    }

    thread::sleep(Duration::from_millis(100));

    // Server should still work with new client
    let request = DaemonRequest::new("42");
    let encoded = request.encode();

    let mut stream = UnixStream::connect(&socket_path).expect("Should connect again");
    stream.write_all(&encoded).expect("Should send");
    stream.flush().expect("Should flush");

    let mut status_buf = [0u8; 1];
    stream
        .read_exact(&mut status_buf)
        .expect("Should read status");

    let mut length_buf = [0u8; 4];
    stream
        .read_exact(&mut length_buf)
        .expect("Should read length");
    let length = u32::from_be_bytes(length_buf) as usize;

    let mut output_buf = vec![0u8; length];
    stream
        .read_exact(&mut output_buf)
        .expect("Should read output");

    let mut full_response = Vec::new();
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) = DaemonResponse::decode(&full_response).expect("Should decode");

    assert!(
        response.is_success(),
        "Server should still work after disconnect"
    );
    assert_eq!(response.output(), "42");

    cleanup_test_files(&socket_path, &pid_path);
}

/// Test protocol integrity: Verify encoded data can survive network transmission
#[test]
fn test_protocol_data_integrity() {
    let test_codes = vec![
        "1+1",
        "x = 100\nx * 2",
        "print('hello')\n42",
        "a = 10\nb = 20\nc = 30\na + b + c",
    ];

    for code in test_codes {
        // Encode request
        let request = DaemonRequest::new(code);
        let encoded = request.encode();

        // Simulate transmission (copy bytes)
        let transmitted = encoded.clone();

        // Decode should work identically
        let (decoded, _) =
            DaemonRequest::decode(&transmitted).expect("Should decode transmitted request");

        assert_eq!(
            decoded.code(),
            code,
            "Transmitted data should match original"
        );
    }
}

/// Test that server handles print statements correctly through full flow
#[test]
fn test_client_server_print_statement_flow() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    let _handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());
    assert!(wait_for_socket(&socket_path, 5), "Server should start");

    let request = DaemonRequest::new("print(10)\nprint(20)\nprint(30)");
    let encoded = request.encode();

    let mut stream = UnixStream::connect(&socket_path).expect("Should connect");
    stream.write_all(&encoded).expect("Should send");
    stream.flush().expect("Should flush");

    // Read response
    let mut status_buf = [0u8; 1];
    stream
        .read_exact(&mut status_buf)
        .expect("Should read status");

    let mut length_buf = [0u8; 4];
    stream
        .read_exact(&mut length_buf)
        .expect("Should read length");
    let length = u32::from_be_bytes(length_buf) as usize;

    let mut output_buf = vec![0u8; length];
    stream
        .read_exact(&mut output_buf)
        .expect("Should read output");

    let mut full_response = Vec::new();
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) = DaemonResponse::decode(&full_response).expect("Should decode");

    assert!(response.is_success(), "Print statements should succeed");
    assert_eq!(
        response.output(),
        "10\n20\n30\n",
        "All print outputs should be captured"
    );

    cleanup_test_files(&socket_path, &pid_path);
}
