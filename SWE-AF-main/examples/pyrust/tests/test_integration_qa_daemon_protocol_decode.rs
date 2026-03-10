//! Integration QA: Daemon Protocol Decode Resolution Verification
//!
//! PRIORITY 1: Tests conflict resolution in DaemonResponse::decode usage
//!
//! Conflicts resolved in:
//! - test_daemon_server.rs: Used '_' for unused decode return value
//! - test_daemon_concurrency.rs: Used '_' for unused decode return value
//!
//! This test verifies that the idiomatic '_' pattern works correctly
//! across all daemon protocol decode scenarios and that the decode
//! functionality is consistent regardless of whether the bytes_consumed
//! return value is used or ignored.

use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};

/// Test that DaemonResponse decode with '_' pattern works for success responses
#[test]
fn test_daemon_response_decode_success_with_ignored_bytes() {
    let request = DaemonRequest::new("2+3");
    let encoded_request = request.encode();

    // Verify request encodes properly
    assert!(
        encoded_request.len() > 0,
        "Request should encode to non-empty bytes"
    );

    // Create a success response
    let response = DaemonResponse::success("5".to_string());
    let encoded = response.encode();

    // Decode using '_' pattern (conflict resolution pattern)
    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode success response");

    assert!(
        decoded.is_success(),
        "Decoded response should indicate success"
    );
    assert_eq!(
        decoded.output(),
        "5",
        "Decoded output should match original"
    );
}

/// Test that DaemonResponse decode with '_' pattern works for error responses
#[test]
fn test_daemon_response_decode_error_with_ignored_bytes() {
    let response = DaemonResponse::error("Division by zero".to_string());
    let encoded = response.encode();

    // Decode using '_' pattern (conflict resolution pattern)
    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode error response");

    assert!(decoded.is_error(), "Decoded response should indicate error");
    assert!(
        decoded.output().contains("Division by zero"),
        "Decoded error should contain message"
    );
}

/// Test that DaemonResponse decode with '_' pattern works for empty output
#[test]
fn test_daemon_response_decode_empty_with_ignored_bytes() {
    let response = DaemonResponse::success("".to_string());
    let encoded = response.encode();

    // Decode using '_' pattern (conflict resolution pattern)
    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode empty response");

    assert!(
        decoded.is_success(),
        "Decoded response should indicate success"
    );
    assert_eq!(decoded.output(), "", "Decoded output should be empty");
}

/// Test that DaemonResponse decode with '_' pattern works for large output
#[test]
fn test_daemon_response_decode_large_output_with_ignored_bytes() {
    let large_output = "x".repeat(10_000);
    let response = DaemonResponse::success(large_output.clone());
    let encoded = response.encode();

    // Decode using '_' pattern (conflict resolution pattern)
    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode large response");

    assert!(
        decoded.is_success(),
        "Decoded response should indicate success"
    );
    assert_eq!(
        decoded.output(),
        large_output,
        "Decoded output should match large input"
    );
}

/// Test that DaemonResponse decode returns correct bytes_consumed when captured
#[test]
fn test_daemon_response_decode_bytes_consumed_accuracy() {
    let response = DaemonResponse::success("42".to_string());
    let encoded = response.encode();
    let encoded_len = encoded.len();

    // Decode and capture bytes_consumed
    let (decoded, bytes_consumed) =
        DaemonResponse::decode(&encoded).expect("Should decode response");

    assert!(decoded.is_success(), "Response should be success");
    assert_eq!(decoded.output(), "42", "Output should be 42");
    assert_eq!(
        bytes_consumed, encoded_len,
        "bytes_consumed should equal encoded length"
    );
}

/// Test decode with extra trailing data (bytes_consumed should be < input length)
#[test]
fn test_daemon_response_decode_with_trailing_data() {
    let response = DaemonResponse::success("test".to_string());
    let mut encoded = response.encode();

    // Add trailing data
    encoded.extend_from_slice(b"TRAILING_GARBAGE");
    let original_encoded_len = encoded.len();

    // Decode should only consume the response bytes
    let (decoded, bytes_consumed) =
        DaemonResponse::decode(&encoded).expect("Should decode despite trailing data");

    assert!(decoded.is_success(), "Response should be success");
    assert_eq!(decoded.output(), "test", "Output should be test");
    assert!(
        bytes_consumed < original_encoded_len,
        "bytes_consumed should be less than total length due to trailing data"
    );
}

/// Test that using '_' vs named variable produces identical decoded results
#[test]
fn test_daemon_response_decode_underscore_vs_named_equivalence() {
    let response = DaemonResponse::error("Test error".to_string());
    let encoded = response.encode();

    // Decode with '_' pattern
    let (decoded1, _) = DaemonResponse::decode(&encoded).expect("Should decode with underscore");

    // Decode with named variable
    let (decoded2, _bytes_consumed) =
        DaemonResponse::decode(&encoded).expect("Should decode with named variable");

    // Both should be identical
    assert_eq!(decoded1.is_success(), decoded2.is_success());
    assert_eq!(decoded1.is_error(), decoded2.is_error());
    assert_eq!(decoded1.output(), decoded2.output());
}

/// Test DaemonRequest encode-decode round trip
#[test]
fn test_daemon_request_encode_decode_roundtrip() {
    let original_code = "x = 10\ny = 20\nx + y";
    let request = DaemonRequest::new(original_code);
    let encoded = request.encode();

    // Decode the request
    let (decoded, _) = DaemonRequest::decode(&encoded).expect("Should decode request");

    assert_eq!(
        decoded.code(),
        original_code,
        "Decoded code should match original"
    );
}

/// Test multiple sequential decodes with '_' pattern
#[test]
fn test_daemon_response_sequential_decodes_with_underscore() {
    let responses = vec![
        ("5", true),
        ("10", true),
        ("Division by zero", false),
        ("42", true),
        ("Undefined variable", false),
    ];

    for (output, is_success) in responses {
        let response = if is_success {
            DaemonResponse::success(output.to_string())
        } else {
            DaemonResponse::error(output.to_string())
        };

        let encoded = response.encode();
        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode response");

        assert_eq!(decoded.is_success(), is_success);
        assert_eq!(decoded.output(), output);
    }
}

/// Test decode failure with invalid data
#[test]
fn test_daemon_response_decode_invalid_data() {
    let invalid_data = vec![0xFF, 0xFF, 0xFF];

    let result = DaemonResponse::decode(&invalid_data);
    assert!(result.is_err(), "Decode should fail with invalid data");
}

/// Test decode with partial data
#[test]
fn test_daemon_response_decode_partial_data() {
    let response = DaemonResponse::success("test".to_string());
    let encoded = response.encode();

    // Take only first few bytes
    let partial = &encoded[..3.min(encoded.len())];

    let result = DaemonResponse::decode(partial);
    assert!(result.is_err(), "Decode should fail with partial data");
}

/// Test that decode is idempotent (decoding encoded data multiple times yields same result)
#[test]
fn test_daemon_response_decode_idempotent() {
    let response = DaemonResponse::success("idempotent".to_string());
    let encoded = response.encode();

    let (decoded1, _) = DaemonResponse::decode(&encoded).expect("First decode should succeed");
    let (decoded2, _) = DaemonResponse::decode(&encoded).expect("Second decode should succeed");
    let (decoded3, _) = DaemonResponse::decode(&encoded).expect("Third decode should succeed");

    assert_eq!(decoded1.output(), decoded2.output());
    assert_eq!(decoded2.output(), decoded3.output());
    assert_eq!(decoded1.is_success(), decoded2.is_success());
    assert_eq!(decoded2.is_success(), decoded3.is_success());
}

/// Test decode with special characters and Unicode
#[test]
fn test_daemon_response_decode_special_characters() {
    let special_outputs = vec![
        "Hello, 世界!",
        "émojis: 🚀🔥💯",
        "newlines:\n\ntabs:\t\t",
        "quotes: \"'`",
        "backslashes: \\ \\\\ \\\\\\",
    ];

    for output in special_outputs {
        let response = DaemonResponse::success(output.to_string());
        let encoded = response.encode();
        let (decoded, _) =
            DaemonResponse::decode(&encoded).expect("Should decode special characters");

        assert_eq!(
            decoded.output(),
            output,
            "Special characters should round-trip correctly"
        );
    }
}
