use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
/// Edge case tests for daemon mode benchmark
///
/// This test suite covers:
/// - Empty input handling
/// - Error conditions (daemon not running, invalid socket)
/// - Boundary conditions (very long code, rapid successive requests)
/// - Connection reuse validation
/// - Statistical stability edge cases
use std::process::Command;
use std::thread;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/pyrust.sock";

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("pyrust");
    path
}

fn start_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = get_binary_path();

    // Clean up any existing socket/PID files
    let _ = fs::remove_file(SOCKET_PATH);
    let _ = fs::remove_file("/tmp/pyrust.pid");

    // Start daemon process
    Command::new(&binary_path).arg("--daemon").spawn()?;

    // Wait for daemon to start (socket should appear)
    for _ in 0..100 {
        if std::path::Path::new(SOCKET_PATH).exists() {
            thread::sleep(Duration::from_millis(50));
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }

    Err("Daemon failed to start within timeout".into())
}

fn stop_daemon() {
    let binary_path = get_binary_path();
    let _ = Command::new(&binary_path).arg("--stop-daemon").output();
    thread::sleep(Duration::from_millis(200));
    let _ = fs::remove_file(SOCKET_PATH);
    let _ = fs::remove_file("/tmp/pyrust.pid");
}

fn send_request_socket(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    send_request_on_stream(&mut stream, code)
}

/// Send a request on an existing Unix socket stream (for connection reuse)
fn send_request_on_stream(
    stream: &mut UnixStream,
    code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let code_bytes = code.as_bytes();
    let length = code_bytes.len() as u32;
    let mut request_bytes = Vec::with_capacity(4 + code_bytes.len());
    request_bytes.extend_from_slice(&length.to_be_bytes());
    request_bytes.extend_from_slice(code_bytes);

    stream.write_all(&request_bytes)?;
    stream.flush()?;

    let mut header_buf = [0u8; 5];
    stream.read_exact(&mut header_buf)?;

    let status = header_buf[0];
    let output_len =
        u32::from_be_bytes([header_buf[1], header_buf[2], header_buf[3], header_buf[4]]) as usize;

    let mut output_buf = vec![0u8; output_len];
    stream.read_exact(&mut output_buf)?;

    let output = String::from_utf8(output_buf)?;

    if status != 0 {
        return Err(format!("Execution error: {}", output).into());
    }

    Ok(output)
}

#[test]
fn test_daemon_mode_empty_input() {
    println!("\n=== Testing Empty Input ===");

    // Build and start daemon
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Test empty code
    let result = send_request_socket("");
    assert!(result.is_ok(), "Empty input should be handled gracefully");
    assert_eq!(
        result.unwrap(),
        "",
        "Empty input should produce empty output"
    );

    stop_daemon();
    println!("✓ Empty input test PASSED");
}

#[test]
fn test_daemon_mode_connection_reuse() {
    println!("\n=== Testing Connection Reuse ===");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Create a single connection and reuse it for multiple requests
    let mut stream = UnixStream::connect(SOCKET_PATH).expect("Failed to connect");

    for i in 0..100 {
        let code = format!("{}+{}", i, i);
        let expected = format!("{}", i + i);

        // Send request on the reused stream
        let result =
            send_request_on_stream(&mut stream, &code).expect(&format!("Request {} failed", i));

        assert_eq!(
            result.trim(),
            expected,
            "Request {} produced wrong result",
            i
        );
    }

    stop_daemon();
    println!("✓ Connection reuse test PASSED (100 requests on single connection)");
}

#[test]
fn test_daemon_mode_rapid_requests() {
    println!("\n=== Testing Rapid Successive Requests ===");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Send requests as fast as possible
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let result = send_request_socket("2+3");
        assert!(result.is_ok(), "Request {} failed", i);
        assert_eq!(
            result.unwrap().trim(),
            "5",
            "Request {} produced wrong result",
            i
        );
    }
    let elapsed = start.elapsed();

    let avg_latency_us = elapsed.as_micros() / 1000;
    println!("1000 requests completed in {:?}", elapsed);
    println!("Average latency: {}μs", avg_latency_us);

    // This should be well under 190μs average
    assert!(
        avg_latency_us < 500,
        "Average latency {}μs is too high (should be <500μs even with new connections)",
        avg_latency_us
    );

    stop_daemon();
    println!("✓ Rapid request test PASSED");
}

#[test]
fn test_daemon_mode_no_daemon_error() {
    println!("\n=== Testing Error When Daemon Not Running ===");

    // Ensure daemon is not running
    stop_daemon();
    thread::sleep(Duration::from_millis(500));

    // Try to connect
    let result = UnixStream::connect(SOCKET_PATH);
    assert!(
        result.is_err(),
        "Should fail to connect when daemon not running"
    );

    println!("✓ Error handling test PASSED");
}

#[test]
fn test_daemon_mode_complex_code() {
    println!("\n=== Testing Complex Code ===");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Test with complex multi-statement code
    let code = "x = 10\ny = 20\nz = x + y\nw = z * 2\nw - 10";
    let result = send_request_socket(code);
    assert!(result.is_ok(), "Complex code should execute");
    assert_eq!(
        result.unwrap().trim(),
        "50",
        "Complex code produced wrong result"
    );

    // Test with print statements
    let code_with_print = "print(42)\nx = 10\nx * 2";
    let result = send_request_socket(code_with_print);
    assert!(result.is_ok(), "Code with print should execute");
    let output = result.unwrap();
    assert!(output.contains("42"), "Should contain print output");
    assert!(output.contains("20"), "Should contain expression result");

    stop_daemon();
    println!("✓ Complex code test PASSED");
}

#[test]
fn test_daemon_mode_error_handling() {
    println!("\n=== Testing Error Handling ===");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Test division by zero
    let result = send_request_socket("10 / 0");
    assert!(result.is_err(), "Division by zero should produce error");

    // Test undefined variable
    let result = send_request_socket("x + 1");
    assert!(result.is_err(), "Undefined variable should produce error");

    // Test syntax error
    let result = send_request_socket("2 + + 3");
    assert!(result.is_err(), "Syntax error should produce error");

    // Verify daemon still works after errors
    let result = send_request_socket("2+3");
    assert!(result.is_ok(), "Daemon should still work after errors");
    assert_eq!(result.unwrap().trim(), "5", "Should produce correct result");

    stop_daemon();
    println!("✓ Error handling test PASSED");
}

#[test]
fn test_daemon_mode_warmup_effect() {
    println!("\n=== Testing Warmup Effect on Latency ===");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Measure first request (cold)
    let start = std::time::Instant::now();
    let _ = send_request_socket("2+3").expect("Failed");
    let cold_latency_us = start.elapsed().as_micros();

    // Warm up with 100 requests
    for _ in 0..100 {
        let _ = send_request_socket("2+3");
    }

    // Measure warm requests
    let mut warm_latencies = Vec::new();
    for _ in 0..100 {
        let start = std::time::Instant::now();
        let _ = send_request_socket("2+3").expect("Failed");
        warm_latencies.push(start.elapsed().as_micros());
    }

    let avg_warm = warm_latencies.iter().sum::<u128>() / warm_latencies.len() as u128;

    println!("Cold latency: {}μs", cold_latency_us);
    println!("Warm average: {}μs", avg_warm);

    // Warm should be faster or comparable
    assert!(
        avg_warm <= cold_latency_us * 2,
        "Warm latency {}μs should not be much worse than cold {}μs",
        avg_warm,
        cold_latency_us
    );

    stop_daemon();
    println!("✓ Warmup effect test PASSED");
}

#[test]
fn test_daemon_mode_concurrent_connections() {
    println!("\n=== Testing Concurrent Connections ===");

    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .status()
        .expect("Failed to build");

    start_daemon().expect("Failed to start daemon");

    // Create multiple connections simultaneously
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..10 {
                    let code = format!("{}+{}", i, j);
                    let expected = format!("{}", i + j);
                    let result = send_request_socket(&code).expect("Request failed");
                    assert_eq!(result.trim(), expected, "Wrong result for {}+{}", i, j);
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    stop_daemon();
    println!("✓ Concurrent connections test PASSED (10 threads × 10 requests)");
}
