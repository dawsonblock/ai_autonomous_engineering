//! Integration QA: Simplified Daemon Client-Server Flow
//!
//! PRIORITY 2: Tests cross-feature interactions (stable subset)
//!
//! Focuses on protocol interactions and client-server contract without
//! relying on timing-sensitive socket operations.

use pyrust::daemon_client::DaemonClient;
use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};

/// Test protocol: Request encode-decode roundtrip
#[test]
fn test_protocol_request_roundtrip() {
    let codes = vec!["2+3", "x = 10\ny = 20\nx + y", "print(42)", ""];

    for code in codes {
        let request = DaemonRequest::new(code);
        let encoded = request.encode();
        let (decoded, _) = DaemonRequest::decode(&encoded).expect("Should decode request");

        assert_eq!(
            decoded.code(),
            code,
            "Request roundtrip should preserve code"
        );
    }
}

/// Test protocol: Response success encode-decode roundtrip
#[test]
fn test_protocol_response_success_roundtrip() {
    let outputs = vec!["5", "30", "42\n", ""];

    for output in outputs {
        let response = DaemonResponse::success(output.to_string());
        let encoded = response.encode();
        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode response");

        assert!(decoded.is_success());
        assert_eq!(
            decoded.output(),
            output,
            "Response roundtrip should preserve output"
        );
    }
}

/// Test protocol: Response error encode-decode roundtrip
#[test]
fn test_protocol_response_error_roundtrip() {
    let errors = vec![
        "Division by zero",
        "Undefined variable: x",
        "ParseError: Expected expression",
    ];

    for error in errors {
        let response = DaemonResponse::error(error.to_string());
        let encoded = response.encode();
        let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should decode error response");

        assert!(decoded.is_error());
        assert_eq!(
            decoded.output(),
            error,
            "Error roundtrip should preserve message"
        );
    }
}

/// Test daemon client fallback execution
#[test]
fn test_daemon_client_fallback_execution() {
    // When no daemon is running, client should fallback to direct execution
    let result = DaemonClient::execute_or_fallback("10 + 20").expect("Fallback should succeed");

    assert_eq!(result, "30", "Fallback should compute correct result");
}

/// Test daemon client fallback with error
#[test]
fn test_daemon_client_fallback_error_handling() {
    let result = DaemonClient::execute_or_fallback("undefined_var");

    assert!(
        result.is_err(),
        "Should return error for undefined variable"
    );
    let error = format!("{}", result.unwrap_err());
    assert!(
        error.contains("Undefined variable"),
        "Error should contain expected message"
    );
}

/// Test daemon client detection (no daemon running)
#[test]
fn test_daemon_client_detection_no_daemon() {
    // Clean up any existing sockets
    let _ = std::fs::remove_file("/tmp/pyrust.sock");

    // Should detect no daemon
    assert!(!DaemonClient::is_daemon_running());

    let status = DaemonClient::daemon_status();
    assert_eq!(status, "Daemon is not running");
}

/// Test protocol consistency: Multiple encodes produce same output
#[test]
fn test_protocol_encode_deterministic() {
    let request = DaemonRequest::new("42");

    let encoded1 = request.encode();
    let encoded2 = request.encode();

    assert_eq!(
        encoded1, encoded2,
        "Multiple encodes should produce identical output"
    );
}

/// Test protocol: Large data handling
#[test]
fn test_protocol_large_data_handling() {
    let large_code = "x = 1\n".repeat(1000);
    let request = DaemonRequest::new(&large_code);
    let encoded = request.encode();
    let (decoded, _) = DaemonRequest::decode(&encoded).expect("Should handle large request");

    assert_eq!(decoded.code(), large_code);
}

/// Test protocol: Unicode and special characters
#[test]
fn test_protocol_unicode_handling() {
    let unicode_cases = vec![
        "print('Hello, 世界!')",
        "# Comment with émojis 🚀",
        "x = 'quotes: \"\\' test'",
    ];

    for code in unicode_cases {
        let request = DaemonRequest::new(code);
        let encoded = request.encode();
        let (decoded, _) = DaemonRequest::decode(&encoded).expect("Should handle Unicode");

        assert_eq!(decoded.code(), code);
    }
}

/// Test client-server contract: Error format consistency
#[test]
fn test_error_format_consistency() {
    // Execute via fallback (no daemon)
    let error1 = DaemonClient::execute_or_fallback("10 / 0");
    assert!(error1.is_err());

    let error2 = DaemonClient::execute_or_fallback("100 / 0");
    assert!(error2.is_err());

    // Both should have consistent error format
    let msg1 = format!("{}", error1.unwrap_err());
    let msg2 = format!("{}", error2.unwrap_err());

    assert!(msg1.contains("Division by zero"));
    assert!(msg2.contains("Division by zero"));
}

/// Test protocol: Empty request/response handling
#[test]
fn test_protocol_empty_handling() {
    // Empty request
    let request = DaemonRequest::new("");
    let encoded = request.encode();
    let (decoded, _) = DaemonRequest::decode(&encoded).expect("Should handle empty request");
    assert_eq!(decoded.code(), "");

    // Empty response
    let response = DaemonResponse::success("".to_string());
    let encoded = response.encode();
    let (decoded, _) = DaemonResponse::decode(&encoded).expect("Should handle empty response");
    assert_eq!(decoded.output(), "");
}

/// Test cross-feature: Client fallback execution with various operations
#[test]
fn test_client_fallback_various_operations() {
    let test_cases = vec![
        ("2+3", "5"),
        ("10*10", "100"),
        ("50-25", "25"),
        ("100 // 3", "33"),
        ("17 % 5", "2"),
    ];

    for (code, expected) in test_cases {
        let result =
            DaemonClient::execute_or_fallback(code).expect(&format!("Should execute: {}", code));

        assert_eq!(
            result, expected,
            "Operation '{}' should return '{}'",
            code, expected
        );
    }
}

/// Test protocol: Bytes consumed accuracy
#[test]
fn test_protocol_bytes_consumed_tracking() {
    let response = DaemonResponse::success("test".to_string());
    let encoded = response.encode();
    let original_len = encoded.len();

    let (_, bytes_consumed) = DaemonResponse::decode(&encoded).expect("Should decode");

    assert_eq!(
        bytes_consumed, original_len,
        "Bytes consumed should equal encoded length"
    );
}

/// Test protocol: Decode only consumes exact bytes needed
#[test]
fn test_protocol_decode_exact_consumption() {
    let response = DaemonResponse::success("data".to_string());
    let mut encoded = response.encode();

    // Add garbage
    encoded.extend_from_slice(b"GARBAGE");
    let total_len = encoded.len();

    let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).expect("Should decode");

    assert_eq!(decoded.output(), "data");
    assert!(
        bytes_consumed < total_len,
        "Should only consume response bytes, not garbage"
    );
}
