//! Binary protocol for daemon IPC
//!
//! This module implements a length-prefixed binary protocol for efficient communication
//! between the PyRust CLI and daemon process via Unix sockets.
//!
//! # Protocol Specification
//!
//! ## Request Format
//! ```text
//! [u32 length (big-endian)][UTF-8 code]
//! ```
//! - `length`: 4-byte big-endian integer indicating the length of the UTF-8 code
//! - `code`: Variable-length UTF-8 encoded Python source code
//!
//! ## Response Format
//! ```text
//! [u8 status][u32 length (big-endian)][UTF-8 output]
//! ```
//! - `status`: 1-byte status code (0 = success, 1 = error)
//! - `length`: 4-byte big-endian integer indicating the length of the UTF-8 output
//! - `output`: Variable-length UTF-8 encoded output or error message
//!
//! # Examples
//!
//! ```
//! use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};
//!
//! // Create and encode a request
//! let request = DaemonRequest::new("2+3");
//! let encoded = request.encode();
//!
//! // Decode the request
//! let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
//! assert_eq!(decoded.code(), "2+3");
//! assert_eq!(bytes_consumed, encoded.len());
//!
//! // Create and encode a success response
//! let response = DaemonResponse::success("5");
//! let encoded = response.encode();
//!
//! // Decode the response
//! let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();
//! assert_eq!(decoded.output(), "5");
//! assert!(decoded.is_success());
//! assert_eq!(bytes_consumed, encoded.len());
//! ```

use std::fmt;

/// Protocol error types
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// Invalid UTF-8 encoding in the message
    InvalidUtf8(String),
    /// Incomplete message (not enough bytes)
    IncompleteMessage(String),
    /// Invalid status code
    InvalidStatus(u8),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidUtf8(msg) => write!(f, "Invalid UTF-8: {}", msg),
            ProtocolError::IncompleteMessage(msg) => write!(f, "Incomplete message: {}", msg),
            ProtocolError::InvalidStatus(status) => write!(f, "Invalid status code: {}", status),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// A daemon request containing Python code to execute
#[derive(Debug, Clone, PartialEq)]
pub struct DaemonRequest {
    code: String,
}

impl DaemonRequest {
    /// Create a new daemon request with the given Python code
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    /// Get the Python code from this request
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Encode the request as a binary message
    ///
    /// Format: [u32 length][UTF-8 code]
    pub fn encode(&self) -> Vec<u8> {
        let code_bytes = self.code.as_bytes();
        let length = code_bytes.len() as u32;

        let mut buffer = Vec::with_capacity(4 + code_bytes.len());
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend_from_slice(code_bytes);

        buffer
    }

    /// Decode a binary message into a daemon request
    ///
    /// Returns `(Self, bytes_consumed)` tuple on success, `ProtocolError` if the message is invalid or incomplete.
    /// The `bytes_consumed` value indicates how many bytes were read from the input slice.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), ProtocolError> {
        // Check we have at least the length prefix
        if bytes.len() < 4 {
            return Err(ProtocolError::IncompleteMessage(format!(
                "Expected at least 4 bytes for length prefix, got {}",
                bytes.len()
            )));
        }

        // Read the length prefix
        let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;

        // Check for integer overflow when computing total message size
        let total_size = 4_usize.checked_add(length).ok_or_else(|| {
            ProtocolError::IncompleteMessage(format!(
                "Length overflow: u32 length {} would overflow usize when adding header",
                length
            ))
        })?;

        // Check we have enough bytes for the code
        if bytes.len() < total_size {
            return Err(ProtocolError::IncompleteMessage(format!(
                "Expected {} bytes of code, got {}",
                length,
                bytes.len() - 4
            )));
        }

        // Extract and validate UTF-8 code
        let code_bytes = &bytes[4..total_size];
        let code = std::str::from_utf8(code_bytes)
            .map_err(|e| ProtocolError::InvalidUtf8(e.to_string()))?
            .to_string();

        Ok((Self { code }, total_size))
    }
}

/// A daemon response containing execution output
#[derive(Debug, Clone, PartialEq)]
pub struct DaemonResponse {
    status: ResponseStatus,
    output: String,
}

/// Response status codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseStatus {
    /// Execution succeeded
    Success = 0,
    /// Execution failed with an error
    Error = 1,
}

impl DaemonResponse {
    /// Create a success response with the given output
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Success,
            output: output.into(),
        }
    }

    /// Create an error response with the given error message
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Error,
            output: error.into(),
        }
    }

    /// Check if this response indicates success
    pub fn is_success(&self) -> bool {
        self.status == ResponseStatus::Success
    }

    /// Check if this response indicates an error
    pub fn is_error(&self) -> bool {
        self.status == ResponseStatus::Error
    }

    /// Get the output or error message from this response
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Encode the response as a binary message
    ///
    /// Format: [u8 status][u32 length][UTF-8 output]
    pub fn encode(&self) -> Vec<u8> {
        let output_bytes = self.output.as_bytes();
        let length = output_bytes.len() as u32;

        let mut buffer = Vec::with_capacity(1 + 4 + output_bytes.len());
        buffer.push(self.status as u8);
        buffer.extend_from_slice(&length.to_be_bytes());
        buffer.extend_from_slice(output_bytes);

        buffer
    }

    /// Decode a binary message into a daemon response
    ///
    /// Returns `(Self, bytes_consumed)` tuple on success, `ProtocolError` if the message is invalid or incomplete.
    /// The `bytes_consumed` value indicates how many bytes were read from the input slice.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), ProtocolError> {
        // Check we have at least the status and length prefix
        if bytes.len() < 5 {
            return Err(ProtocolError::IncompleteMessage(format!(
                "Expected at least 5 bytes for status and length prefix, got {}",
                bytes.len()
            )));
        }

        // Read the status byte
        let status = match bytes[0] {
            0 => ResponseStatus::Success,
            1 => ResponseStatus::Error,
            other => return Err(ProtocolError::InvalidStatus(other)),
        };

        // Read the length prefix
        let length = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;

        // Check for integer overflow when computing total message size
        let total_size = 5_usize.checked_add(length).ok_or_else(|| {
            ProtocolError::IncompleteMessage(format!(
                "Length overflow: u32 length {} would overflow usize when adding header",
                length
            ))
        })?;

        // Check we have enough bytes for the output
        if bytes.len() < total_size {
            return Err(ProtocolError::IncompleteMessage(format!(
                "Expected {} bytes of output, got {}",
                length,
                bytes.len() - 5
            )));
        }

        // Extract and validate UTF-8 output
        let output_bytes = &bytes[5..total_size];
        let output = std::str::from_utf8(output_bytes)
            .map_err(|e| ProtocolError::InvalidUtf8(e.to_string()))?
            .to_string();

        Ok((Self { status, output }, total_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_request_encode_decode_empty() {
        let request = DaemonRequest::new("");
        let encoded = request.encode();
        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), "");
        assert_eq!(bytes_consumed, 4); // Only header
    }

    #[test]
    fn test_request_encode_decode_simple() {
        let request = DaemonRequest::new("2+3");
        let encoded = request.encode();
        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), "2+3");
        assert_eq!(bytes_consumed, 7); // 4-byte header + 3 bytes
    }

    #[test]
    fn test_request_encode_decode_large_payload() {
        // Create a 1MB payload
        let large_code = "x = 1\n".repeat(1024 * 1024 / 6); // ~1MB
        let request = DaemonRequest::new(large_code.clone());
        let encoded = request.encode();
        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), large_code);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_request_encode_format() {
        let request = DaemonRequest::new("2+3");
        let encoded = request.encode();

        // Check format: [u32 length][UTF-8 code]
        assert_eq!(encoded.len(), 4 + 3);

        // Check length prefix (big-endian)
        let length = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        assert_eq!(length, 3);

        // Check code
        let code = std::str::from_utf8(&encoded[4..]).unwrap();
        assert_eq!(code, "2+3");
    }

    #[test]
    fn test_request_decode_invalid_utf8() {
        // Create invalid UTF-8 sequence
        let mut bytes = vec![0, 0, 0, 3]; // length = 3
        bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // invalid UTF-8

        let result = DaemonRequest::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::InvalidUtf8(_) => {}
            other => panic!("Expected InvalidUtf8, got {:?}", other),
        }
    }

    #[test]
    fn test_request_decode_incomplete_message() {
        // Only length prefix, no code
        let bytes = vec![0, 0, 0, 10]; // length = 10, but no code

        let result = DaemonRequest::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }
    }

    #[test]
    fn test_request_decode_no_length_prefix() {
        // Less than 4 bytes
        let bytes = vec![0, 0];

        let result = DaemonRequest::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }
    }

    #[test]
    fn test_response_encode_decode_success() {
        let response = DaemonResponse::success("5");
        let encoded = response.encode();
        let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();

        assert!(decoded.is_success());
        assert!(!decoded.is_error());
        assert_eq!(decoded.output(), "5");
        assert_eq!(bytes_consumed, 6); // 1-byte status + 4-byte header + 1 byte
    }

    #[test]
    fn test_response_encode_decode_error() {
        let response = DaemonResponse::error("Division by zero");
        let encoded = response.encode();
        let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();

        assert!(decoded.is_error());
        assert!(!decoded.is_success());
        assert_eq!(decoded.output(), "Division by zero");
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_response_encode_format() {
        let response = DaemonResponse::success("42");
        let encoded = response.encode();

        // Check format: [u8 status][u32 length][UTF-8 output]
        assert_eq!(encoded.len(), 1 + 4 + 2);

        // Check status
        assert_eq!(encoded[0], 0);

        // Check length prefix (big-endian)
        let length = u32::from_be_bytes([encoded[1], encoded[2], encoded[3], encoded[4]]);
        assert_eq!(length, 2);

        // Check output
        let output = std::str::from_utf8(&encoded[5..]).unwrap();
        assert_eq!(output, "42");
    }

    #[test]
    fn test_response_decode_invalid_utf8() {
        // Create response with invalid UTF-8
        let mut bytes = vec![0]; // status = success
        bytes.extend_from_slice(&[0, 0, 0, 3]); // length = 3
        bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // invalid UTF-8

        let result = DaemonResponse::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::InvalidUtf8(_) => {}
            other => panic!("Expected InvalidUtf8, got {:?}", other),
        }
    }

    #[test]
    fn test_response_decode_incomplete_message() {
        // Only status and length prefix, no output
        let bytes = vec![0, 0, 0, 0, 10]; // status + length = 10, but no output

        let result = DaemonResponse::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }
    }

    #[test]
    fn test_response_decode_no_status() {
        // Less than 5 bytes
        let bytes = vec![0, 0];

        let result = DaemonResponse::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }
    }

    #[test]
    fn test_response_decode_invalid_status() {
        // Invalid status code (not 0 or 1)
        let mut bytes = vec![99]; // invalid status
        bytes.extend_from_slice(&[0, 0, 0, 2]); // length = 2
        bytes.extend_from_slice(b"ok");

        let result = DaemonResponse::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::InvalidStatus(99) => {}
            other => panic!("Expected InvalidStatus(99), got {:?}", other),
        }
    }

    #[test]
    fn test_round_trip_request() {
        let large_code = "a".repeat(1000);
        let test_cases = vec![
            "",
            "2+3",
            "print(42)",
            "x = 10\ny = 20\nz = x + y\nprint(z)\nz",
            &large_code, // 1KB
        ];

        for code in test_cases {
            let request = DaemonRequest::new(code);
            let encoded = request.encode();
            let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
            assert_eq!(decoded.code(), code);
            assert_eq!(bytes_consumed, encoded.len());
        }
    }

    #[test]
    fn test_round_trip_response() {
        let large_output = "x".repeat(1000);
        let test_cases = vec![
            ("", true),
            ("5", true),
            ("42", true),
            ("Division by zero", false),
            ("RuntimeError: undefined variable", false),
            (&large_output, true), // 1KB
        ];

        for (output, is_success) in test_cases {
            let response = if is_success {
                DaemonResponse::success(output)
            } else {
                DaemonResponse::error(output)
            };

            let encoded = response.encode();
            let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();

            assert_eq!(decoded.output(), output);
            assert_eq!(decoded.is_success(), is_success);
            assert_eq!(decoded.is_error(), !is_success);
            assert_eq!(bytes_consumed, encoded.len());
        }
    }

    #[test]
    fn test_encode_performance_request() {
        let request = DaemonRequest::new("2+3");

        // Warm up to ensure consistent timing
        for _ in 0..10 {
            let _ = request.encode();
        }

        let start = Instant::now();
        let _encoded = request.encode();
        let duration = start.elapsed();

        // In release mode this should be < 5μs, but debug mode is slower
        // We verify it completes (correctness) and print timing for informational purposes
        println!("Request encode took {:?}", duration);

        // Sanity check: should be reasonably fast even in debug mode (< 1ms)
        assert!(
            duration.as_micros() < 1000,
            "Encode took {:?}, expected < 1ms",
            duration
        );
    }

    #[test]
    fn test_decode_performance_request() {
        let request = DaemonRequest::new("2+3");
        let encoded = request.encode();

        // Warm up
        for _ in 0..10 {
            let _ = DaemonRequest::decode(&encoded).unwrap();
        }

        let start = Instant::now();
        let (_decoded, _) = DaemonRequest::decode(&encoded).unwrap();
        let duration = start.elapsed();

        // In release mode this should be < 10μs, but debug mode is slower
        println!("Request decode took {:?}", duration);

        // Sanity check: should be reasonably fast even in debug mode (< 1ms)
        assert!(
            duration.as_micros() < 1000,
            "Decode took {:?}, expected < 1ms",
            duration
        );
    }

    #[test]
    fn test_encode_performance_response() {
        let response = DaemonResponse::success("42");

        // Warm up
        for _ in 0..10 {
            let _ = response.encode();
        }

        let start = Instant::now();
        let _encoded = response.encode();
        let duration = start.elapsed();

        // In release mode this should be < 5μs, but debug mode is slower
        println!("Response encode took {:?}", duration);

        // Sanity check: should be reasonably fast even in debug mode (< 1ms)
        assert!(
            duration.as_micros() < 1000,
            "Encode took {:?}, expected < 1ms",
            duration
        );
    }

    #[test]
    fn test_decode_performance_response() {
        let response = DaemonResponse::success("42");
        let encoded = response.encode();

        // Warm up
        for _ in 0..10 {
            let _ = DaemonResponse::decode(&encoded).unwrap();
        }

        let start = Instant::now();
        let (_decoded, _) = DaemonResponse::decode(&encoded).unwrap();
        let duration = start.elapsed();

        // In release mode this should be < 10μs, but debug mode is slower
        println!("Response decode took {:?}", duration);

        // Sanity check: should be reasonably fast even in debug mode (< 1ms)
        assert!(
            duration.as_micros() < 1000,
            "Decode took {:?}, expected < 1ms",
            duration
        );
    }

    #[test]
    fn test_large_payload_1mb() {
        // Create a 1MB payload to test performance
        let large_code = "x = 1\n".repeat(1024 * 1024 / 6);
        let request = DaemonRequest::new(&large_code);

        // Test encode
        let start = Instant::now();
        let encoded = request.encode();
        let encode_duration = start.elapsed();

        // Test decode
        let start = Instant::now();
        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        let decode_duration = start.elapsed();

        assert_eq!(decoded.code(), large_code);
        assert_eq!(bytes_consumed, encoded.len());

        // Print performance metrics (informational, not strict requirement)
        println!("1MB encode: {:?}", encode_duration);
        println!("1MB decode: {:?}", decode_duration);
    }

    #[test]
    fn test_unicode_handling() {
        // Test various Unicode characters
        let test_cases = vec![
            "print('Hello, 世界')",
            "x = 'émoji: 🚀'",
            "# Comment with Ü and ñ",
        ];

        for code in test_cases {
            let request = DaemonRequest::new(code);
            let encoded = request.encode();
            let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
            assert_eq!(decoded.code(), code);
            assert_eq!(bytes_consumed, encoded.len());
        }
    }

    #[test]
    fn test_empty_code() {
        let request = DaemonRequest::new("");
        let encoded = request.encode();

        // Should have 4-byte length prefix with value 0
        assert_eq!(encoded.len(), 4);
        assert_eq!(
            u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]),
            0
        );

        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), "");
        assert_eq!(bytes_consumed, 4);
    }

    #[test]
    fn test_empty_output() {
        let response = DaemonResponse::success("");
        let encoded = response.encode();

        // Should have 1-byte status + 4-byte length prefix with value 0
        assert_eq!(encoded.len(), 5);
        assert_eq!(encoded[0], 0); // success status
        assert_eq!(
            u32::from_be_bytes([encoded[1], encoded[2], encoded[3], encoded[4]]),
            0
        );

        let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();
        assert_eq!(decoded.output(), "");
        assert!(decoded.is_success());
        assert_eq!(bytes_consumed, 5);
    }

    // Additional edge case tests for comprehensive coverage

    #[test]
    fn test_request_max_u32_length() {
        // Test with maximum representable length value
        // This tests boundary condition of u32 length encoding
        let long_code = "a".repeat(100000); // 100KB (large but not maximum)
        let request = DaemonRequest::new(&long_code);
        let encoded = request.encode();

        // Verify length encoding
        let length = u32::from_be_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        assert_eq!(length, 100000);

        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), long_code);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_response_with_newlines() {
        // Test that responses correctly handle multiline output
        let multiline = "line1\nline2\nline3";
        let response = DaemonResponse::success(multiline);
        let encoded = response.encode();
        let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();

        assert_eq!(decoded.output(), multiline);
        assert!(decoded.is_success());
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_request_decode_extra_bytes() {
        // Test that decode ignores trailing bytes after valid message
        let request = DaemonRequest::new("test");
        let mut encoded = request.encode();
        let expected_consumed = encoded.len();
        encoded.extend_from_slice(b"extra bytes"); // Add extra data

        // Should successfully decode the valid portion
        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), "test");
        // Bytes consumed should be only the valid message, not the extra bytes
        assert_eq!(bytes_consumed, expected_consumed);
        assert_eq!(bytes_consumed, 8); // 4-byte header + 4 bytes "test"
    }

    #[test]
    fn test_response_decode_extra_bytes() {
        // Test that decode ignores trailing bytes after valid message
        let response = DaemonResponse::success("result");
        let mut encoded = response.encode();
        let expected_consumed = encoded.len();
        encoded.extend_from_slice(b"garbage"); // Add extra data

        // Should successfully decode the valid portion
        let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();
        assert_eq!(decoded.output(), "result");
        // Bytes consumed should be only the valid message, not the extra bytes
        assert_eq!(bytes_consumed, expected_consumed);
        assert_eq!(bytes_consumed, 11); // 1-byte status + 4-byte header + 6 bytes "result"
    }

    #[test]
    fn test_request_special_characters() {
        // Test encoding/decoding of special characters and escape sequences
        let special = "tab\there\nnewline\rcarriage\x00null";
        let request = DaemonRequest::new(special);
        let encoded = request.encode();
        let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), special);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_response_error_status_byte() {
        // Verify error status is encoded as 1
        let response = DaemonResponse::error("error message");
        let encoded = response.encode();

        assert_eq!(encoded[0], 1); // error status should be 1

        let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();
        assert!(decoded.is_error());
        assert!(!decoded.is_success());
        assert_eq!(decoded.output(), "error message");
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_request_decode_length_mismatch() {
        // Test when length prefix doesn't match actual data length
        let mut bytes = vec![0, 0, 0, 5]; // length says 5
        bytes.extend_from_slice(b"ab"); // but only 2 bytes provided

        let result = DaemonRequest::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }
    }

    #[test]
    fn test_response_decode_length_mismatch() {
        // Test when length prefix doesn't match actual data length
        let mut bytes = vec![0]; // status = success
        bytes.extend_from_slice(&[0, 0, 0, 10]); // length says 10
        bytes.extend_from_slice(b"short"); // but only 5 bytes provided

        let result = DaemonResponse::decode(&bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }
    }

    #[test]
    fn test_request_clone_equality() {
        // Test that DaemonRequest supports clone and equality
        let request1 = DaemonRequest::new("test code");
        let request2 = request1.clone();

        assert_eq!(request1, request2);
        assert_eq!(request1.code(), request2.code());
    }

    #[test]
    fn test_response_clone_equality() {
        // Test that DaemonResponse supports clone and equality
        let response1 = DaemonResponse::success("output");
        let response2 = response1.clone();

        assert_eq!(response1, response2);
        assert_eq!(response1.output(), response2.output());
        assert_eq!(response1.is_success(), response2.is_success());
    }

    #[test]
    fn test_protocol_error_display() {
        // Test that protocol errors have meaningful display messages
        let utf8_err = ProtocolError::InvalidUtf8("test error".to_string());
        let incomplete_err = ProtocolError::IncompleteMessage("need more bytes".to_string());
        let status_err = ProtocolError::InvalidStatus(99);

        assert!(utf8_err.to_string().contains("Invalid UTF-8"));
        assert!(incomplete_err.to_string().contains("Incomplete message"));
        assert!(status_err.to_string().contains("Invalid status code"));
        assert!(status_err.to_string().contains("99"));
    }

    #[test]
    fn test_zero_length_request() {
        // Test explicit zero-length encoding
        let bytes = vec![0, 0, 0, 0]; // length = 0, no code

        let (decoded, bytes_consumed) = DaemonRequest::decode(&bytes).unwrap();
        assert_eq!(decoded.code(), "");
        assert_eq!(bytes_consumed, 4);
    }

    #[test]
    fn test_zero_length_response() {
        // Test explicit zero-length encoding
        let mut bytes = vec![0]; // status = success
        bytes.extend_from_slice(&[0, 0, 0, 0]); // length = 0, no output

        let (decoded, bytes_consumed) = DaemonResponse::decode(&bytes).unwrap();
        assert_eq!(decoded.output(), "");
        assert!(decoded.is_success());
        assert_eq!(bytes_consumed, 5);
    }

    // New tests for bytes consumed validation and streaming protocol support

    #[test]
    fn test_request_bytes_consumed_streaming() {
        // Test that bytes_consumed allows streaming protocol usage
        // Simulate a stream with multiple messages
        let req1 = DaemonRequest::new("first");
        let req2 = DaemonRequest::new("second");

        let mut stream = Vec::new();
        stream.extend_from_slice(&req1.encode());
        stream.extend_from_slice(&req2.encode());

        // Decode first message
        let (decoded1, consumed1) = DaemonRequest::decode(&stream).unwrap();
        assert_eq!(decoded1.code(), "first");
        assert_eq!(consumed1, 9); // 4-byte header + 5 bytes "first"

        // Decode second message from remaining bytes
        let (decoded2, consumed2) = DaemonRequest::decode(&stream[consumed1..]).unwrap();
        assert_eq!(decoded2.code(), "second");
        assert_eq!(consumed2, 10); // 4-byte header + 6 bytes "second"

        // Total consumed should match stream length
        assert_eq!(consumed1 + consumed2, stream.len());
    }

    #[test]
    fn test_response_bytes_consumed_streaming() {
        // Test that bytes_consumed allows streaming protocol usage
        // Simulate a stream with multiple messages
        let resp1 = DaemonResponse::success("ok");
        let resp2 = DaemonResponse::error("fail");

        let mut stream = Vec::new();
        stream.extend_from_slice(&resp1.encode());
        stream.extend_from_slice(&resp2.encode());

        // Decode first message
        let (decoded1, consumed1) = DaemonResponse::decode(&stream).unwrap();
        assert_eq!(decoded1.output(), "ok");
        assert!(decoded1.is_success());
        assert_eq!(consumed1, 7); // 1-byte status + 4-byte header + 2 bytes "ok"

        // Decode second message from remaining bytes
        let (decoded2, consumed2) = DaemonResponse::decode(&stream[consumed1..]).unwrap();
        assert_eq!(decoded2.output(), "fail");
        assert!(decoded2.is_error());
        assert_eq!(consumed2, 9); // 1-byte status + 4-byte header + 4 bytes "fail"

        // Total consumed should match stream length
        assert_eq!(consumed1 + consumed2, stream.len());
    }

    #[test]
    fn test_request_bytes_consumed_exact_match() {
        // Test that bytes_consumed exactly matches encoded length for various sizes
        let x100 = "x".repeat(100);
        let y1000 = "y".repeat(1000);
        let test_cases = vec![
            ("", 4),
            ("a", 5),
            ("ab", 6),
            ("hello", 9),
            (x100.as_str(), 104),
            (y1000.as_str(), 1004),
        ];

        for (code, expected_size) in test_cases {
            let request = DaemonRequest::new(code);
            let encoded = request.encode();
            assert_eq!(
                encoded.len(),
                expected_size,
                "Encoded size mismatch for code length {}",
                code.len()
            );

            let (decoded, bytes_consumed) = DaemonRequest::decode(&encoded).unwrap();
            assert_eq!(decoded.code(), code);
            assert_eq!(
                bytes_consumed,
                expected_size,
                "Bytes consumed mismatch for code length {}",
                code.len()
            );
            assert_eq!(
                bytes_consumed,
                encoded.len(),
                "Bytes consumed should match encoded length"
            );
        }
    }

    #[test]
    fn test_response_bytes_consumed_exact_match() {
        // Test that bytes_consumed exactly matches encoded length for various sizes
        let x100 = "x".repeat(100);
        let y1000 = "y".repeat(1000);
        let test_cases = vec![
            ("", 5),
            ("a", 6),
            ("ab", 7),
            ("hello", 10),
            (x100.as_str(), 105),
            (y1000.as_str(), 1005),
        ];

        for (output, expected_size) in test_cases {
            let response = DaemonResponse::success(output);
            let encoded = response.encode();
            assert_eq!(
                encoded.len(),
                expected_size,
                "Encoded size mismatch for output length {}",
                output.len()
            );

            let (decoded, bytes_consumed) = DaemonResponse::decode(&encoded).unwrap();
            assert_eq!(decoded.output(), output);
            assert_eq!(
                bytes_consumed,
                expected_size,
                "Bytes consumed mismatch for output length {}",
                output.len()
            );
            assert_eq!(
                bytes_consumed,
                encoded.len(),
                "Bytes consumed should match encoded length"
            );
        }
    }

    #[test]
    fn test_request_integer_overflow_protection() {
        // Test that decode properly handles potential integer overflow
        // Create a message with max u32 length that would overflow when adding header size
        // Note: We can't actually create a 4GB buffer in tests, so we test the check logic
        // by crafting a message with a length that would cause overflow

        // On 32-bit systems, u32::MAX + 4 would overflow usize
        // We simulate this by creating a message that claims to have a very large length
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&u32::MAX.to_be_bytes()); // length = u32::MAX
                                                          // Don't add actual data - just test the overflow check

        let result = DaemonRequest::decode(&bytes);
        // On 64-bit systems, this won't overflow but will fail due to incomplete message
        // On 32-bit systems (if any), this would catch the overflow
        assert!(result.is_err());
    }

    #[test]
    fn test_response_integer_overflow_protection() {
        // Test that decode properly handles potential integer overflow for responses
        let mut bytes = Vec::new();
        bytes.push(0); // status = success
        bytes.extend_from_slice(&u32::MAX.to_be_bytes()); // length = u32::MAX
                                                          // Don't add actual data - just test the overflow check

        let result = DaemonResponse::decode(&bytes);
        // On 64-bit systems, this won't overflow but will fail due to incomplete message
        // On 32-bit systems (if any), this would catch the overflow
        assert!(result.is_err());
    }

    #[test]
    fn test_request_partial_buffer_streaming() {
        // Test realistic streaming scenario where buffer might have partial messages
        let request = DaemonRequest::new("test123");
        let encoded = request.encode();

        // Try to decode with only partial buffer (just the header)
        let result = DaemonRequest::decode(&encoded[..4]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }

        // Try with header + partial data
        let result = DaemonRequest::decode(&encoded[..8]); // Only 4 of 7 data bytes
        assert!(result.is_err());
        match result.unwrap_err() {
            ProtocolError::IncompleteMessage(_) => {}
            other => panic!("Expected IncompleteMessage, got {:?}", other),
        }

        // Full message should work
        let (decoded, consumed) = DaemonRequest::decode(&encoded).unwrap();
        assert_eq!(decoded.code(), "test123");
        assert_eq!(consumed, encoded.len());
    }

    #[test]
    fn test_response_partial_buffer_streaming() {
        // Test realistic streaming scenario where buffer might have partial messages
        let response = DaemonResponse::success("result");
        let encoded = response.encode();

        // Try to decode with only status byte
        let result = DaemonResponse::decode(&encoded[..1]);
        assert!(result.is_err());

        // Try with status + partial header
        let result = DaemonResponse::decode(&encoded[..4]);
        assert!(result.is_err());

        // Try with status + header but partial data
        let result = DaemonResponse::decode(&encoded[..7]); // Only 2 of 6 data bytes
        assert!(result.is_err());

        // Full message should work
        let (decoded, consumed) = DaemonResponse::decode(&encoded).unwrap();
        assert_eq!(decoded.output(), "result");
        assert_eq!(consumed, encoded.len());
    }
}
