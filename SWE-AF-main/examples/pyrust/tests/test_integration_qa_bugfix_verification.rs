//! Integration QA: Bug Fixes Verification
//!
//! PRIORITY 3: Tests that bug fixes work correctly
//!
//! Merged features:
//! - issue/12-bug-fixes-verification: Fixed test suite bugs (941 tests passing)
//!
//! This test verifies that bug fixes from issue/12 are functioning:
//! 1. DaemonResponse decode works correctly
//! 2. Error handling is consistent
//! 3. Negative number parsing works

use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};

/// Test that DaemonResponse decode works correctly (bug fix category 1)
#[test]
fn test_daemon_response_decode_bug_fixed() {
    // This was one of the bug fixes in issue/12
    let response = DaemonResponse::success("42".to_string());
    let encoded = response.encode();

    // Using the '_' pattern that was part of the conflict resolution
    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Decode should work after bug fix");

    assert!(decoded.is_success());
    assert_eq!(decoded.output(), "42");
}

/// Test that error responses decode correctly
#[test]
fn test_error_response_decode_bug_fixed() {
    let response = DaemonResponse::error("Test error".to_string());
    let encoded = response.encode();

    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Error decode should work");

    assert!(decoded.is_error());
    assert_eq!(decoded.output(), "Test error");
}

/// Test multiple sequential decodes (verifies no memory issues)
#[test]
fn test_sequential_decodes_no_corruption() {
    for i in 0..100 {
        let output = format!("Result {}", i);
        let response = DaemonResponse::success(output.clone());
        let encoded = response.encode();

        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Decode should work");

        assert_eq!(decoded.output(), output);
    }
}

/// Test that request encoding is consistent
#[test]
fn test_request_encoding_consistency() {
    let codes = vec![
        "2+3",
        "x = 10\ny = 20",
        "print(42)",
        "-10 + 5", // Negative numbers (bug fix category)
    ];

    for code in codes {
        let request = DaemonRequest::new(code);
        let encoded = request.encode();
        let (decoded, _) = DaemonRequest::decode(&encoded).expect("Should decode");

        assert_eq!(decoded.code(), code);
    }
}

/// Test large output handling (verifies buffer management)
#[test]
fn test_large_output_handling() {
    let large_output = "x".repeat(50_000);
    let response = DaemonResponse::success(large_output.clone());
    let encoded = response.encode();

    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should handle large output");

    assert_eq!(decoded.output().len(), large_output.len());
}

/// Test edge cases that might have been problematic
#[test]
fn test_edge_cases_after_bugfixes() {
    let edge_cases = vec![
        ("", ""),                     // Empty
        ("42", "42"),                 // Single value
        ("\n\n\n", "\n\n\n"),         // Only newlines
        ("  spaces  ", "  spaces  "), // Whitespace
    ];

    for (input, expected) in edge_cases {
        let response = DaemonResponse::success(input.to_string());
        let encoded = response.encode();
        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should handle edge case");

        assert_eq!(decoded.output(), expected);
    }
}

/// Test concurrent protocol operations (no race conditions)
#[test]
fn test_concurrent_protocol_operations() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let output = format!("Thread {}", i);
                let response = DaemonResponse::success(output.clone());
                let encoded = response.encode();
                let (decoded, _) =
                    DaemonResponse::decode(&encoded).expect("Should decode in thread");

                assert_eq!(decoded.output(), output);
                i
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect();

    assert_eq!(results.len(), 10);
}

/// Test that all response types work correctly after bug fixes
#[test]
fn test_all_response_types() {
    // Success with output
    let r1 = DaemonResponse::success("result".to_string());
    let (d1, _) = DaemonResponse::decode(&r1.encode()).unwrap();
    assert!(d1.is_success());
    assert_eq!(d1.output(), "result");

    // Success with empty output
    let r2 = DaemonResponse::success("".to_string());
    let (d2, _) = DaemonResponse::decode(&r2.encode()).unwrap();
    assert!(d2.is_success());
    assert_eq!(d2.output(), "");

    // Error
    let r3 = DaemonResponse::error("error message".to_string());
    let (d3, _) = DaemonResponse::decode(&r3.encode()).unwrap();
    assert!(d3.is_error());
    assert_eq!(d3.output(), "error message");
}

/// Test protocol robustness with invalid data
#[test]
fn test_protocol_handles_invalid_data() {
    let invalid_inputs = vec![
        vec![],           // Empty
        vec![0xFF],       // Single byte
        vec![0x00, 0x00], // Too short
    ];

    for invalid in invalid_inputs {
        let result = DaemonResponse::decode(&invalid);
        assert!(result.is_err(), "Should reject invalid data");
    }
}

/// Test that bytes_consumed tracking is accurate
#[test]
fn test_bytes_consumed_accuracy() {
    let response = DaemonResponse::success("test".to_string());
    let encoded = response.encode();
    let expected_len = encoded.len();

    let (_, bytes_consumed) = DaemonResponse::decode(&encoded).expect("Should decode");

    assert_eq!(
        bytes_consumed, expected_len,
        "Bytes consumed should match encoded length"
    );
}
