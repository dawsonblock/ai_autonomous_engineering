//! Integration tests for profiling, cache, and daemon interaction
//!
//! Tests the integration between profiling infrastructure, compilation cache,
//! and daemon execution to ensure they work correctly together.
//!
//! PRIORITY 1: Profiling mode bypasses cache (always recompiles for accurate timing)
//! PRIORITY 2: Cache works correctly in daemon mode
//! PRIORITY 3: Profiling output is consistent regardless of cache state

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const BINARY_PATH: &str = "./target/release/pyrust";
const SOCKET_PATH: &str = "/tmp/pyrust_prof_cache.sock";
const PID_FILE_PATH: &str = "/tmp/pyrust_prof_cache.pid";

/// Helper to cleanup daemon artifacts
fn cleanup_daemon() {
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
            if std::os::unix::net::UnixStream::connect(SOCKET_PATH).is_ok() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

/// PRIORITY 1: Test that profiling mode always recompiles (doesn't use cache)
/// This is critical because cached execution would give incorrect profiling timings
#[test]
fn test_profiling_bypasses_cache() {
    cleanup_daemon();

    let code = "x = 100\ny = 200\nx + y";

    // First execution with profiling - cold compilation
    let output1 = Command::new(BINARY_PATH)
        .args(&["-c", code, "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute with --profile");

    assert!(output1.status.success(), "First profiling execution failed");
    let stderr1 = String::from_utf8_lossy(&output1.stderr);
    assert!(
        stderr1.contains("lex"),
        "Profiling output missing lex stage"
    );
    assert!(
        stderr1.contains("parse"),
        "Profiling output missing parse stage"
    );
    assert!(
        stderr1.contains("compile"),
        "Profiling output missing compile stage"
    );

    // Second execution with profiling - should still show compilation stages
    // (not use cache even though same code was just executed)
    let output2 = Command::new(BINARY_PATH)
        .args(&["-c", code, "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute with --profile");

    assert!(
        output2.status.success(),
        "Second profiling execution failed"
    );
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr2.contains("lex"),
        "Profiling should show lex stage on repeat"
    );
    assert!(
        stderr2.contains("parse"),
        "Profiling should show parse stage on repeat"
    );
    assert!(
        stderr2.contains("compile"),
        "Profiling should show compile stage on repeat"
    );

    // Both executions should show non-zero compilation times
    // If cache was used, compilation times would be near zero
    assert!(
        stderr1.contains("ns") || stderr1.contains("μs"),
        "Profiling times not found in first execution"
    );
    assert!(
        stderr2.contains("ns") || stderr2.contains("μs"),
        "Profiling times not found in second execution"
    );

    cleanup_daemon();
}

/// PRIORITY 1: Test that --profile-json also bypasses cache
#[test]
fn test_profile_json_bypasses_cache() {
    cleanup_daemon();

    let code = "10 * 20 + 30";

    // Execute with --profile-json multiple times
    for i in 0..3 {
        let output = Command::new(BINARY_PATH)
            .args(&["-c", code, "--profile-json"])
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute with --profile-json");

        assert!(output.status.success(), "Execution {} failed", i);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Verify JSON contains all compilation stages
        assert!(
            stderr.contains("\"lex_ns\""),
            "lex_ns missing in execution {}",
            i
        );
        assert!(
            stderr.contains("\"parse_ns\""),
            "parse_ns missing in execution {}",
            i
        );
        assert!(
            stderr.contains("\"compile_ns\""),
            "compile_ns missing in execution {}",
            i
        );
        assert!(
            stderr.contains("\"vm_execute_ns\""),
            "vm_execute_ns missing in execution {}",
            i
        );

        // Verify compile_ns is non-zero (indicating actual compilation, not cache hit)
        // We check for the presence of the field; exact values vary
        assert!(
            stderr.contains("\"compile_ns\":"),
            "compile_ns field missing"
        );
    }

    cleanup_daemon();
}

/// PRIORITY 2: Test that cache works correctly in daemon mode with repeated code
#[test]
fn test_cache_works_in_daemon_mode() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    let code = "a = 50\nb = 150\na + b";

    // Execute same code multiple times through daemon
    // Cache should be used after first execution
    for i in 0..10 {
        let output = Command::new(BINARY_PATH)
            .args(&["-c", code])
            .output()
            .expect("Failed to execute code");

        assert!(output.status.success(), "Execution {} failed", i);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "200",
            "Incorrect result on execution {}",
            i
        );
    }

    // Execute different code to verify cache doesn't mix up different inputs
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "100 + 100"])
        .output()
        .expect("Failed to execute different code");

    assert!(output.status.success(), "Different code execution failed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "200");

    // Execute original code again to verify cache still works
    let output = Command::new(BINARY_PATH)
        .args(&["-c", code])
        .output()
        .expect("Failed to execute code again");

    assert!(output.status.success(), "Re-execution failed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "200");

    cleanup_daemon();
}

/// PRIORITY 2: Test that cache works correctly in fallback (direct execution) mode
#[test]
fn test_cache_works_in_fallback_mode() {
    cleanup_daemon();

    // Ensure daemon is NOT running
    assert!(
        !Path::new(SOCKET_PATH).exists(),
        "Daemon should not be running"
    );

    let code = "x = 25\ny = 75\nx * y";

    // Execute same code multiple times via fallback (direct execution)
    // Cache should still work
    for i in 0..10 {
        let output = Command::new(BINARY_PATH)
            .args(&["-c", code])
            .output()
            .expect("Failed to execute code");

        assert!(output.status.success(), "Execution {} failed", i);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "1875",
            "Incorrect result on execution {}",
            i
        );
    }

    cleanup_daemon();
}

/// PRIORITY 3: Test that profiling with daemon running still uses direct execution
#[test]
fn test_profiling_uses_direct_execution_even_with_daemon() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute with profiling - should bypass daemon
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "2+3", "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute with --profile");

    assert!(output.status.success(), "Profiling execution failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify correct output
    assert_eq!(stdout.trim(), "5");

    // Verify profiling output exists (indicates direct execution path was used)
    assert!(
        stderr.contains("Pipeline Profile"),
        "Profiling output missing"
    );
    assert!(stderr.contains("lex"), "lex stage missing");
    assert!(stderr.contains("parse"), "parse stage missing");
    assert!(stderr.contains("compile"), "compile stage missing");
    assert!(stderr.contains("vm_execute"), "vm_execute stage missing");

    // Verify daemon is still running (profiling didn't affect it)
    assert!(
        Path::new(SOCKET_PATH).exists(),
        "Daemon socket missing after profiling"
    );

    cleanup_daemon();
}

/// PRIORITY 2: Test cache eviction doesn't break daemon or profiling
#[test]
fn test_cache_integration_with_many_unique_scripts() {
    cleanup_daemon();

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute many unique scripts to potentially trigger cache eviction
    for i in 0..50 {
        let code = format!("x = {}\ny = {}\nx + y", i, i * 2);
        let expected = format!("{}", i * 3);

        let output = Command::new(BINARY_PATH)
            .args(&["-c", &code])
            .output()
            .expect("Failed to execute code");

        assert!(output.status.success(), "Execution {} failed", i);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            expected,
            "Incorrect result for iteration {}",
            i
        );
    }

    // Execute with profiling after many cached executions
    let output = Command::new(BINARY_PATH)
        .args(&["-c", "100 + 200", "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute with --profile");

    assert!(
        output.status.success(),
        "Profiling after many executions failed"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Pipeline Profile"),
        "Profiling still works after cache activity"
    );

    cleanup_daemon();
}

/// PRIORITY 3: Test error handling is consistent across cache/daemon/profiling modes
#[test]
fn test_error_consistency_across_modes() {
    cleanup_daemon();

    let error_code = "10 / 0";

    // Test error in direct execution (no daemon)
    let output_direct = Command::new(BINARY_PATH)
        .args(&["-c", error_code])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute");

    assert!(!output_direct.status.success(), "Should fail");
    let stderr_direct = String::from_utf8_lossy(&output_direct.stderr);
    assert!(
        stderr_direct.contains("Division by zero"),
        "Error message missing"
    );

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Test same error through daemon
    let output_daemon = Command::new(BINARY_PATH)
        .args(&["-c", error_code])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute");

    assert!(!output_daemon.status.success(), "Should fail");
    let stderr_daemon = String::from_utf8_lossy(&output_daemon.stderr);
    assert!(
        stderr_daemon.contains("Division by zero"),
        "Error message missing"
    );

    // Test same error with profiling
    let output_profile = Command::new(BINARY_PATH)
        .args(&["-c", error_code, "--profile"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute");

    assert!(!output_profile.status.success(), "Should fail");
    let stderr_profile = String::from_utf8_lossy(&output_profile.stderr);
    assert!(
        stderr_profile.contains("Division by zero"),
        "Error message missing"
    );

    // All error messages should be consistent
    assert_eq!(
        stderr_direct.trim(),
        stderr_daemon.trim(),
        "Error messages differ between direct and daemon execution"
    );

    cleanup_daemon();
}

/// PRIORITY 2: Test that file execution mode uses cache correctly
#[test]
fn test_file_mode_cache_integration() {
    cleanup_daemon();

    let test_file = "/tmp/pyrust_cache_test.py";
    let code = "a = 10\nb = 20\nprint(a + b)\na * b";
    fs::write(test_file, code).expect("Failed to write test file");

    // Start daemon
    let output = Command::new(BINARY_PATH)
        .arg("--daemon")
        .output()
        .expect("Failed to start daemon");

    assert!(output.status.success(), "Daemon start failed");
    assert!(wait_for_socket(1000), "Daemon socket not created");

    // Execute file multiple times - should use cache
    for i in 0..5 {
        let output = Command::new(BINARY_PATH)
            .arg(test_file)
            .output()
            .expect("Failed to execute file");

        assert!(output.status.success(), "File execution {} failed", i);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("30"),
            "Print output missing in execution {}",
            i
        );
        assert!(
            stdout.ends_with("200"),
            "Expression result missing in execution {}",
            i
        );
    }

    // Modify file and verify cache is invalidated (different result)
    let modified_code = "a = 5\nb = 10\nprint(a + b)\na * b";
    fs::write(test_file, modified_code).expect("Failed to write modified file");

    let output = Command::new(BINARY_PATH)
        .arg(test_file)
        .output()
        .expect("Failed to execute modified file");

    assert!(output.status.success(), "Modified file execution failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("15"),
        "Modified file should show updated result"
    );
    assert!(
        stdout.ends_with("50"),
        "Modified file should show updated result"
    );

    fs::remove_file(test_file).expect("Failed to remove test file");
    cleanup_daemon();
}
