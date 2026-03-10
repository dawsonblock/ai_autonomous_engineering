//! Unix socket daemon server for PyRust
//!
//! This module implements a Unix socket daemon server that accepts connections
//! and executes Python code via the daemon_protocol. It provides:
//!
//! - Unix socket event loop at /tmp/pyrust.sock
//! - Signal handling (SIGTERM/SIGINT) for graceful shutdown
//! - PID file management at /tmp/pyrust.pid
//! - Request timeout to prevent hung connections
//! - Socket permissions set to 0600 (owner only)
//!
//! # Example
//!
//! ```no_run
//! use pyrust::daemon::DaemonServer;
//!
//! let daemon = DaemonServer::new().unwrap();
//! daemon.run().unwrap();
//! ```

use crate::daemon_protocol::{DaemonRequest, DaemonResponse, ProtocolError};
use crate::execute_python_cached_global;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Default socket path
pub const SOCKET_PATH: &str = "/tmp/pyrust.sock";

/// Default PID file path
pub const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

/// Request timeout in seconds
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Maximum request size (10 MB)
const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;

/// Daemon server error types
#[derive(Debug)]
pub enum DaemonError {
    /// IO error
    Io(std::io::Error),
    /// Protocol error
    Protocol(ProtocolError),
    /// Socket already in use
    SocketInUse(String),
    /// PID file error
    PidFileError(String),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonError::Io(e) => write!(f, "IO error: {}", e),
            DaemonError::Protocol(e) => write!(f, "Protocol error: {}", e),
            DaemonError::SocketInUse(path) => write!(f, "Socket already in use: {}", path),
            DaemonError::PidFileError(msg) => write!(f, "PID file error: {}", msg),
        }
    }
}

impl std::error::Error for DaemonError {}

impl From<std::io::Error> for DaemonError {
    fn from(e: std::io::Error) -> Self {
        DaemonError::Io(e)
    }
}

impl From<ProtocolError> for DaemonError {
    fn from(e: ProtocolError) -> Self {
        DaemonError::Protocol(e)
    }
}

/// Unix socket daemon server
pub struct DaemonServer {
    socket_path: String,
    pid_file_path: String,
    shutdown_flag: Arc<AtomicBool>,
}

impl DaemonServer {
    /// Create a new daemon server with default paths
    pub fn new() -> Result<Self, DaemonError> {
        Self::with_paths(SOCKET_PATH.to_string(), PID_FILE_PATH.to_string())
    }

    /// Create a new daemon server with custom paths
    pub fn with_paths(socket_path: String, pid_file_path: String) -> Result<Self, DaemonError> {
        // Check if socket already exists
        if Path::new(&socket_path).exists() {
            // Try to connect to check if daemon is running
            if UnixStream::connect(&socket_path).is_ok() {
                return Err(DaemonError::SocketInUse(socket_path));
            }
            // Socket exists but no daemon listening - remove stale socket
            fs::remove_file(&socket_path)?;
        }

        let shutdown_flag = Arc::new(AtomicBool::new(false));

        // Setup signal handlers
        Self::setup_signal_handlers(Arc::clone(&shutdown_flag));

        Ok(Self {
            socket_path,
            pid_file_path,
            shutdown_flag,
        })
    }

    /// Setup signal handlers for SIGTERM and SIGINT
    fn setup_signal_handlers(shutdown_flag: Arc<AtomicBool>) {
        // Create signal handler for SIGTERM
        let shutdown_flag_term = Arc::clone(&shutdown_flag);
        unsafe {
            signal_hook::low_level::register(signal_hook::consts::SIGTERM, move || {
                shutdown_flag_term.store(true, Ordering::SeqCst);
            })
            .expect("Failed to register SIGTERM handler");
        }

        // Create signal handler for SIGINT
        let shutdown_flag_int = Arc::clone(&shutdown_flag);
        unsafe {
            signal_hook::low_level::register(signal_hook::consts::SIGINT, move || {
                shutdown_flag_int.store(true, Ordering::SeqCst);
            })
            .expect("Failed to register SIGINT handler");
        }
    }

    /// Write PID file
    fn write_pid_file(&self) -> Result<(), DaemonError> {
        let pid = std::process::id();

        // If PID file exists, check if the process is still running
        if Path::new(&self.pid_file_path).exists() {
            if let Ok(old_pid_str) = fs::read_to_string(&self.pid_file_path) {
                if let Ok(_old_pid) = old_pid_str.trim().parse::<u32>() {
                    // For now, just remove the old PID file
                    // In production, we would check if the process is running
                    let _ = fs::remove_file(&self.pid_file_path);
                }
            }
        }

        fs::write(&self.pid_file_path, pid.to_string())
            .map_err(|e| DaemonError::PidFileError(format!("Failed to write PID file: {}", e)))?;
        Ok(())
    }

    /// Remove PID file
    fn remove_pid_file(&self) {
        let _ = fs::remove_file(&self.pid_file_path);
    }

    /// Run the daemon server
    pub fn run(&self) -> Result<(), DaemonError> {
        // Bind to Unix socket
        let listener = UnixListener::bind(&self.socket_path)?;

        // Set socket permissions to 0600 (owner only)
        let metadata = fs::metadata(&self.socket_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&self.socket_path, permissions)?;

        // Write PID file
        self.write_pid_file()?;

        // Set non-blocking mode for the listener to check shutdown flag
        listener.set_nonblocking(true)?;

        // Event loop
        loop {
            // Check shutdown flag
            if self.shutdown_flag.load(Ordering::SeqCst) {
                break;
            }

            // Accept connection (non-blocking)
            match listener.accept() {
                Ok((stream, _addr)) => {
                    // Handle connection
                    if let Err(e) = self.handle_connection(stream) {
                        eprintln!("Error handling connection: {}", e);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, sleep briefly and check shutdown flag again
                    std::thread::sleep(Duration::from_micros(100));
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }

        // Cleanup
        self.cleanup()?;

        Ok(())
    }

    /// Handle a client connection (supports multiple requests on same connection)
    fn handle_connection(&self, mut stream: UnixStream) -> Result<(), DaemonError> {
        // Ensure socket is in blocking mode (listener is non-blocking but streams should block)
        stream.set_nonblocking(false)?;

        // Set idle timeout for persistent connections (5 seconds between requests)
        // This allows connection reuse for fast clients (benchmarks) while not blocking
        // new connections for too long
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(REQUEST_TIMEOUT_SECS)))?;

        // Handle multiple requests on same connection until client closes or idle timeout
        loop {
            // Read request (will return error when client closes or timeout)
            let request = match self.read_request(&mut stream) {
                Ok(req) => req,
                Err(DaemonError::Io(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Client closed connection gracefully
                    break;
                }
                Err(DaemonError::Io(ref e))
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    // Idle timeout - no request received in 5 seconds, close connection
                    break;
                }
                Err(e) => return Err(e),
            };

            // Execute code using global cache (shared across all daemon requests)
            let response = match execute_python_cached_global(request.code()) {
                Ok(output) => DaemonResponse::success(output),
                Err(e) => DaemonResponse::error(e.to_string()),
            };

            // Send response
            self.write_response(&mut stream, &response)?;
        }

        Ok(())
    }

    /// Read a request from the stream
    fn read_request(&self, stream: &mut UnixStream) -> Result<DaemonRequest, DaemonError> {
        // Read length prefix (4 bytes)
        let mut length_buf = [0u8; 4];
        stream.read_exact(&mut length_buf)?;
        let length = u32::from_be_bytes(length_buf) as usize;

        // Check size limit
        if length > MAX_REQUEST_SIZE {
            return Err(DaemonError::Protocol(ProtocolError::IncompleteMessage(
                format!(
                    "Request too large: {} bytes (max {})",
                    length, MAX_REQUEST_SIZE
                ),
            )));
        }

        // Read code
        let mut code_buf = vec![0u8; length];
        stream.read_exact(&mut code_buf)?;

        // Reconstruct full message and decode
        let mut full_message = Vec::with_capacity(4 + length);
        full_message.extend_from_slice(&length_buf);
        full_message.extend_from_slice(&code_buf);

        let (request, _bytes_consumed) = DaemonRequest::decode(&full_message)?;
        Ok(request)
    }

    /// Write a response to the stream
    fn write_response(
        &self,
        stream: &mut UnixStream,
        response: &DaemonResponse,
    ) -> Result<(), DaemonError> {
        let encoded = response.encode();
        stream.write_all(&encoded)?;
        stream.flush()?;
        Ok(())
    }

    /// Cleanup resources (socket and PID file)
    fn cleanup(&self) -> Result<(), DaemonError> {
        // Remove socket
        if Path::new(&self.socket_path).exists() {
            fs::remove_file(&self.socket_path)?;
        }

        // Remove PID file
        self.remove_pid_file();

        Ok(())
    }

    /// Stop the daemon (for testing)
    pub fn stop(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }
}

impl Drop for DaemonServer {
    fn drop(&mut self) {
        // Ensure cleanup on drop
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_error_display() {
        let err = DaemonError::SocketInUse("/tmp/test.sock".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Socket already in use"));
        assert!(display.contains("/tmp/test.sock"));
    }

    #[test]
    fn test_daemon_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let daemon_err: DaemonError = io_err.into();
        assert!(matches!(daemon_err, DaemonError::Io(_)));
    }

    #[test]
    fn test_daemon_error_from_protocol_error() {
        let protocol_err = ProtocolError::InvalidUtf8("test".to_string());
        let daemon_err: DaemonError = protocol_err.into();
        assert!(matches!(daemon_err, DaemonError::Protocol(_)));
    }

    #[test]
    fn test_socket_permissions_value() {
        // Verify that 0o600 = owner read+write only
        assert_eq!(0o600, 0o600);
        assert_eq!(0o600 & 0o400, 0o400); // owner read
        assert_eq!(0o600 & 0o200, 0o200); // owner write
        assert_eq!(0o600 & 0o100, 0o000); // no owner execute
        assert_eq!(0o600 & 0o070, 0o000); // no group permissions
        assert_eq!(0o600 & 0o007, 0o000); // no other permissions
    }

    #[test]
    fn test_max_request_size_constant() {
        assert_eq!(MAX_REQUEST_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_request_timeout_constant() {
        assert_eq!(REQUEST_TIMEOUT_SECS, 30);
    }

    #[test]
    fn test_default_paths() {
        assert_eq!(SOCKET_PATH, "/tmp/pyrust.sock");
        assert_eq!(PID_FILE_PATH, "/tmp/pyrust.pid");
    }
}
