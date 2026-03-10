//! Daemon client for communicating with the PyRust daemon server
//!
//! This module provides client-side functionality for connecting to a Unix socket daemon,
//! sending code execution requests, and receiving results. It includes automatic fallback
//! to direct execution if the daemon is unavailable.
//!
//! # Architecture
//!
//! The daemon client implements a simple request-response pattern over Unix sockets:
//! 1. Check if daemon is running (socket file exists)
//! 2. Connect to Unix socket
//! 3. Send code execution request using binary protocol
//! 4. Receive response with result or error
//! 5. Fall back to direct execution if daemon unavailable
//!
//! # Example
//!
//! ```no_run
//! use pyrust::daemon_client::DaemonClient;
//!
//! // Execute code with automatic fallback
//! let result = DaemonClient::execute_or_fallback("2+3").unwrap();
//! assert_eq!(result, "5");
//!
//! // Check daemon status
//! if DaemonClient::is_daemon_running() {
//!     println!("Daemon is running");
//! }
//!
//! // Stop the daemon
//! DaemonClient::stop_daemon().unwrap();
//! ```

use std::fmt;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::daemon_protocol::{DaemonRequest, DaemonResponse};
use crate::execute_python;

/// Unix socket path for daemon IPC
pub const SOCKET_PATH: &str = "/tmp/pyrust.sock";

/// PID file path for daemon process tracking
pub const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

/// Maximum response size (10MB) to prevent unbounded allocation
const MAX_RESPONSE_SIZE: usize = 10_485_760;

/// Client interface for daemon communication
pub struct DaemonClient;

impl DaemonClient {
    /// Check if daemon is running by testing socket existence
    ///
    /// # Returns
    ///
    /// `true` if the socket file exists, indicating a daemon is likely running
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pyrust::daemon_client::DaemonClient;
    ///
    /// if DaemonClient::is_daemon_running() {
    ///     println!("Daemon is running");
    /// } else {
    ///     println!("Daemon is not running");
    /// }
    /// ```
    pub fn is_daemon_running() -> bool {
        Path::new(SOCKET_PATH).exists()
    }

    /// Execute code via daemon with automatic fallback to direct execution
    ///
    /// This method attempts to send the code to the daemon for execution. If the
    /// daemon is unavailable or any communication error occurs, it automatically
    /// falls back to direct execution via `execute_python()`.
    ///
    /// # Arguments
    ///
    /// * `code` - Python source code to execute
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Execution output (either from daemon or direct execution)
    /// * `Err(Box<dyn std::error::Error>)` - Error from direct execution (only if daemon unavailable)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pyrust::daemon_client::DaemonClient;
    ///
    /// let result = DaemonClient::execute_or_fallback("2+3").unwrap();
    /// assert_eq!(result, "5");
    /// ```
    pub fn execute_or_fallback(code: &str) -> Result<String, Box<dyn std::error::Error>> {
        match Self::execute_via_daemon(code) {
            Ok(output) => Ok(output),
            Err(_) => {
                // Daemon unavailable, fallback to direct execution
                execute_python(code).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            }
        }
    }

    /// Execute code via daemon connection
    ///
    /// This is a private method that handles the actual communication with the daemon.
    /// It sends a request, waits for a response, and returns the result or error.
    ///
    /// # Arguments
    ///
    /// * `code` - Python source code to execute
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Execution output from daemon
    /// * `Err(DaemonClientError)` - Communication or execution error
    fn execute_via_daemon(code: &str) -> Result<String, DaemonClientError> {
        // Connect to Unix socket with timeout
        let mut stream =
            UnixStream::connect(SOCKET_PATH).map_err(DaemonClientError::ConnectionFailed)?;

        // Set timeouts for read/write to prevent hung requests
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(DaemonClientError::SocketConfig)?;
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .map_err(DaemonClientError::SocketConfig)?;

        // Encode and send request using binary protocol
        let request = DaemonRequest::new(code);
        let request_bytes = request.encode();

        stream
            .write_all(&request_bytes)
            .map_err(DaemonClientError::WriteFailed)?;
        stream.flush().map_err(DaemonClientError::WriteFailed)?;

        // Read response header (status + length = 5 bytes)
        let mut header_buf = [0u8; 5];
        stream
            .read_exact(&mut header_buf)
            .map_err(DaemonClientError::ReadFailed)?;

        // Parse response length
        let output_len =
            u32::from_be_bytes([header_buf[1], header_buf[2], header_buf[3], header_buf[4]])
                as usize;

        // Safety check: prevent unbounded allocation
        if output_len > MAX_RESPONSE_SIZE {
            return Err(DaemonClientError::ResponseTooLarge {
                size: output_len,
                max: MAX_RESPONSE_SIZE,
            });
        }

        // Read response body
        let mut output_buf = vec![0u8; output_len];
        stream
            .read_exact(&mut output_buf)
            .map_err(DaemonClientError::ReadFailed)?;

        // Combine header and body for decoding
        let mut full_response = Vec::with_capacity(5 + output_len);
        full_response.extend_from_slice(&header_buf);
        full_response.extend_from_slice(&output_buf);

        // Decode response
        let (response, _bytes_consumed) = DaemonResponse::decode(&full_response)
            .map_err(|e| DaemonClientError::ProtocolError(format!("{}", e)))?;

        // Check status and return result or error
        if response.is_success() {
            Ok(response.output().to_string())
        } else {
            // Return execution error with the error message from daemon
            Err(DaemonClientError::ExecutionError(
                response.output().to_string(),
            ))
        }
    }

    /// Stop running daemon by reading PID and sending SIGTERM
    ///
    /// This method reads the daemon's PID from the PID file, sends a SIGTERM signal,
    /// waits briefly for cleanup, and verifies that the socket file has been removed.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Daemon stopped successfully
    /// * `Err(DaemonClientError)` - Failed to stop daemon or verify cleanup
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pyrust::daemon_client::DaemonClient;
    ///
    /// DaemonClient::stop_daemon().unwrap();
    /// ```
    pub fn stop_daemon() -> Result<(), DaemonClientError> {
        use std::fs;

        // Read PID from file
        let pid_str = fs::read_to_string(PID_FILE_PATH).map_err(DaemonClientError::PidFileRead)?;

        let pid: i32 = pid_str
            .trim()
            .parse()
            .map_err(|e| DaemonClientError::InvalidPid(format!("{}", e)))?;

        // Send SIGTERM to daemon process
        #[cfg(unix)]
        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }

        // Wait briefly for cleanup
        std::thread::sleep(Duration::from_millis(100));

        // Verify shutdown by checking socket file removal
        if Path::new(SOCKET_PATH).exists() {
            return Err(DaemonClientError::ShutdownFailed);
        }

        Ok(())
    }

    /// Get daemon status as a human-readable string
    ///
    /// # Returns
    ///
    /// A string indicating whether the daemon is running or not
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pyrust::daemon_client::DaemonClient;
    ///
    /// println!("{}", DaemonClient::daemon_status());
    /// ```
    pub fn daemon_status() -> String {
        if Self::is_daemon_running() {
            "Daemon is running".to_string()
        } else {
            "Daemon is not running".to_string()
        }
    }
}

/// Errors that can occur during daemon client operations
#[derive(Debug)]
pub enum DaemonClientError {
    /// Failed to connect to daemon socket
    ConnectionFailed(std::io::Error),
    /// Failed to configure socket options (timeouts, etc.)
    SocketConfig(std::io::Error),
    /// Failed to write request to socket
    WriteFailed(std::io::Error),
    /// Failed to read response from socket
    ReadFailed(std::io::Error),
    /// Invalid UTF-8 in response
    InvalidUtf8(std::string::FromUtf8Error),
    /// Invalid status code in response
    InvalidStatus(u8),
    /// Execution error returned by daemon
    ExecutionError(String),
    /// Response size exceeds maximum allowed
    ResponseTooLarge { size: usize, max: usize },
    /// Failed to read PID file
    PidFileRead(std::io::Error),
    /// Invalid PID value in file
    InvalidPid(String),
    /// Daemon failed to shutdown properly
    ShutdownFailed,
    /// Protocol error during decode
    ProtocolError(String),
}

impl fmt::Display for DaemonClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonClientError::ConnectionFailed(e) => {
                write!(f, "Failed to connect to daemon: {}", e)
            }
            DaemonClientError::SocketConfig(e) => write!(f, "Failed to configure socket: {}", e),
            DaemonClientError::WriteFailed(e) => write!(f, "Failed to write to daemon: {}", e),
            DaemonClientError::ReadFailed(e) => write!(f, "Failed to read from daemon: {}", e),
            DaemonClientError::InvalidUtf8(e) => write!(f, "Invalid UTF-8 in response: {}", e),
            DaemonClientError::InvalidStatus(s) => write!(f, "Invalid status code: {}", s),
            DaemonClientError::ExecutionError(msg) => write!(f, "{}", msg),
            DaemonClientError::ResponseTooLarge { size, max } => {
                write!(f, "Response too large: {} bytes (max {})", size, max)
            }
            DaemonClientError::PidFileRead(e) => write!(f, "Failed to read PID file: {}", e),
            DaemonClientError::InvalidPid(msg) => write!(f, "Invalid PID: {}", msg),
            DaemonClientError::ShutdownFailed => write!(f, "Daemon failed to shutdown cleanly"),
            DaemonClientError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for DaemonClientError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    // Mutex to serialize tests that manipulate the socket file
    static SOCKET_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_is_daemon_running_when_socket_missing() {
        let _lock = SOCKET_TEST_LOCK.lock().unwrap();
        // Ensure socket doesn't exist before and after test
        let _ = fs::remove_file(SOCKET_PATH);

        let result = DaemonClient::is_daemon_running();

        // Cleanup any stray files
        let _ = fs::remove_file(SOCKET_PATH);

        assert!(!result);
    }

    #[test]
    fn test_is_daemon_running_when_socket_exists() {
        let _lock = SOCKET_TEST_LOCK.lock().unwrap();

        // Create a temporary socket file
        let _ = fs::remove_file(SOCKET_PATH);
        fs::write(SOCKET_PATH, "").expect("Failed to create test socket file");

        let result = DaemonClient::is_daemon_running();

        // Cleanup
        fs::remove_file(SOCKET_PATH).expect("Failed to cleanup test socket file");

        assert!(result);
    }

    #[test]
    fn test_daemon_status_not_running() {
        let _lock = SOCKET_TEST_LOCK.lock().unwrap();

        // Ensure socket doesn't exist
        let _ = fs::remove_file(SOCKET_PATH);

        let status = DaemonClient::daemon_status();

        // Cleanup any stray files
        let _ = fs::remove_file(SOCKET_PATH);

        assert_eq!(status, "Daemon is not running");
    }

    #[test]
    fn test_daemon_status_running() {
        let _lock = SOCKET_TEST_LOCK.lock().unwrap();

        // Create a temporary socket file
        let _ = fs::remove_file(SOCKET_PATH);
        fs::write(SOCKET_PATH, "").expect("Failed to create test socket file");

        let status = DaemonClient::daemon_status();

        // Cleanup
        fs::remove_file(SOCKET_PATH).expect("Failed to cleanup test socket file");

        assert_eq!(status, "Daemon is running");
    }

    #[test]
    fn test_error_display() {
        let err = DaemonClientError::ConnectionFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "socket not found",
        ));
        let display = format!("{}", err);
        assert!(display.contains("Failed to connect to daemon"));

        let err = DaemonClientError::ResponseTooLarge {
            size: 20_000_000,
            max: 10_485_760,
        };
        let display = format!("{}", err);
        assert!(display.contains("Response too large"));
        assert!(display.contains("20000000"));
    }
}
