use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// Socket path for daemon communication
const SOCKET_PATH: &str = "/tmp/pyrust.sock";

/// PID file path for daemon process tracking
const PID_FILE_PATH: &str = "/tmp/pyrust.pid";

/// Get the path to the release binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("pyrust");
    path
}

/// Start the daemon server in the background
fn start_daemon() -> Result<Child, Box<dyn std::error::Error>> {
    let binary_path = get_binary_path();

    // Ensure binary exists
    if !binary_path.exists() {
        return Err(format!(
            "Binary not found at {:?}. Run 'cargo build --release' first.",
            binary_path
        )
        .into());
    }

    // Clean up any existing socket/PID files from previous runs
    let _ = fs::remove_file(SOCKET_PATH);
    let _ = fs::remove_file(PID_FILE_PATH);

    // Start daemon process
    let child = Command::new(&binary_path)
        .arg("--daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Wait for daemon to initialize (socket file should appear)
    for _ in 0..100 {
        if std::path::Path::new(SOCKET_PATH).exists() {
            // Give daemon a bit more time to fully initialize
            thread::sleep(Duration::from_millis(50));
            return Ok(child);
        }
        thread::sleep(Duration::from_millis(10));
    }

    Err("Daemon failed to start within timeout".into())
}

/// Stop the daemon server
fn stop_daemon() {
    let binary_path = get_binary_path();

    // Send stop command
    let _ = Command::new(&binary_path).arg("--stop-daemon").output();

    // Wait for cleanup
    thread::sleep(Duration::from_millis(200));

    // Force cleanup of socket/PID files if they still exist
    let _ = fs::remove_file(SOCKET_PATH);
    let _ = fs::remove_file(PID_FILE_PATH);
}

/// Execute code via daemon using CLI binary (includes process spawn overhead)
fn execute_via_daemon_cli(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let binary_path = get_binary_path();

    let output = Command::new(&binary_path).arg("-c").arg(code).output()?;

    if !output.status.success() {
        return Err(format!(
            "Daemon execution failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Execute code via daemon using direct Unix socket communication (no process spawn)
/// This measures pure daemon server latency without client process overhead
/// NOTE: Creates a new connection per call - use send_request_reuse for benchmarking
fn execute_via_daemon_socket(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Connect to Unix socket
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    send_request_on_stream(&mut stream, code)
}

/// Send a request on an existing Unix socket stream (for connection reuse)
/// This eliminates socket handshake overhead and measures pure daemon latency
fn send_request_on_stream(
    stream: &mut UnixStream,
    code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Encode request: [u32 length][UTF-8 code]
    let code_bytes = code.as_bytes();
    let length = code_bytes.len() as u32;
    let mut request_bytes = Vec::with_capacity(4 + code_bytes.len());
    request_bytes.extend_from_slice(&length.to_be_bytes());
    request_bytes.extend_from_slice(code_bytes);

    // Send request
    stream.write_all(&request_bytes)?;
    stream.flush()?;

    // Read response header: [u8 status][u32 length]
    let mut header_buf = [0u8; 5];
    stream.read_exact(&mut header_buf)?;

    let status = header_buf[0];
    let output_len =
        u32::from_be_bytes([header_buf[1], header_buf[2], header_buf[3], header_buf[4]]) as usize;

    // Read response body
    let mut output_buf = vec![0u8; output_len];
    stream.read_exact(&mut output_buf)?;

    let output = String::from_utf8(output_buf)?;

    if status != 0 {
        return Err(format!("Execution error: {}", output).into());
    }

    Ok(output)
}

/// Warm up the daemon with several requests to ensure stable performance
fn warmup_daemon() {
    // Connect once and reuse connection for warmup (matches validation script methodology)
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for warmup");

    // Perform 1000 warmup requests to stabilize daemon performance and cache
    for i in 0..1000 {
        let code = format!("{}", i % 100);
        let _ = send_request_on_stream(&mut stream, &code);
    }

    // Also warm up the specific code patterns we'll benchmark
    for _ in 0..10 {
        let _ = send_request_on_stream(&mut stream, "2+3");
        let _ = send_request_on_stream(&mut stream, "(10 + 20) * 3 / 2");
    }

    // Give system time to stabilize
    thread::sleep(Duration::from_millis(100));
}

/// Benchmark: Daemon mode per-request latency for simple arithmetic (AC6.2 - target ≤190μs)
/// This measures pure daemon server latency using direct Unix socket communication
/// without client process spawn overhead
fn bench_daemon_mode_simple_arithmetic(c: &mut Criterion) {
    // Start daemon once for all iterations
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    // Warm up daemon
    warmup_daemon();

    // Create socket connection OUTSIDE iteration loop to eliminate handshake overhead
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_simple_arithmetic", |b| {
        b.iter(|| {
            let output = send_request_on_stream(&mut stream, black_box("2+3"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "5", "Unexpected output: {}", output);
        });
    });

    // Stop daemon after benchmark completes
    stop_daemon();
}

/// Benchmark: Daemon mode for complex expression
fn bench_daemon_mode_complex_expression(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_complex_expression", |b| {
        b.iter(|| {
            let output = send_request_on_stream(&mut stream, black_box("(10 + 20) * 3 / 2"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "45", "Unexpected output: {}", output);
        });
    });

    stop_daemon();
}

/// Benchmark: Daemon mode with variables
fn bench_daemon_mode_with_variables(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_with_variables", |b| {
        b.iter(|| {
            let output = send_request_on_stream(&mut stream, black_box("x = 10\ny = 20\nx + y"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "30", "Unexpected output: {}", output);
        });
    });

    stop_daemon();
}

/// Benchmark: Daemon mode with print statement
fn bench_daemon_mode_print(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_with_print", |b| {
        b.iter(|| {
            let output = send_request_on_stream(&mut stream, black_box("print(42)"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "42", "Unexpected output: {}", output);
        });
    });

    stop_daemon();
}

/// Benchmark: Daemon mode with multiple operations
fn bench_daemon_mode_multiple_operations(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_multiple_operations", |b| {
        b.iter(|| {
            let output = send_request_on_stream(&mut stream, black_box("10 + 5 * 2 - 8 / 4 % 3"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "18", "Unexpected output: {}", output);
        });
    });

    stop_daemon();
}

/// Benchmark: Daemon mode by code complexity
fn bench_daemon_mode_by_complexity(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    let mut group = c.benchmark_group("daemon_mode_by_complexity");

    let test_cases = vec![
        ("minimal", "42", "42"),
        ("simple_arithmetic", "2+3", "5"),
        ("medium_expression", "(10 + 20) * 3", "90"),
        ("complex_program", "x = 10\ny = 20\nz = x + y\nz * 2", "60"),
    ];

    for (name, code, expected) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, &code| {
            b.iter(|| {
                let output = send_request_on_stream(&mut stream, black_box(code))
                    .expect("Failed to execute via daemon socket");
                assert_eq!(output.trim(), expected, "Unexpected output: {}", output);
            });
        });
    }

    group.finish();
    stop_daemon();
}

/// Benchmark: Daemon mode measuring cache effectiveness
/// This uses identical code to test cache hit performance
fn bench_daemon_mode_cache_hit(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    // Pre-warm the cache with the specific code we'll benchmark
    for _ in 0..10 {
        let _ = send_request_on_stream(&mut stream, "2+3");
    }

    c.bench_function("daemon_mode_cache_hit", |b| {
        b.iter(|| {
            let output = send_request_on_stream(&mut stream, black_box("2+3"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "5", "Unexpected output: {}", output);
        });
    });

    stop_daemon();
}

/// Benchmark: Daemon mode throughput test (1000 sequential requests)
/// This simulates the M2 acceptance criteria validation scenario
fn bench_daemon_mode_throughput(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_1000_requests", |b| {
        b.iter(|| {
            // Execute requests measuring average latency
            // Criterion will handle the iteration count, so we just do one request per iter
            let output = send_request_on_stream(&mut stream, black_box("2+3"))
                .expect("Failed to execute via daemon socket");
            assert_eq!(output.trim(), "5");
        });
    });

    stop_daemon();
}

/// Benchmark: Daemon mode minimal overhead (empty program)
fn bench_daemon_mode_minimal_overhead(c: &mut Criterion) {
    let mut _daemon_process = start_daemon().expect("Failed to start daemon for benchmark");

    warmup_daemon();

    // Reuse socket connection for all iterations
    let mut stream =
        UnixStream::connect(SOCKET_PATH).expect("Failed to connect to daemon for benchmark");

    c.bench_function("daemon_mode_minimal_overhead", |b| {
        b.iter(|| {
            let _ = send_request_on_stream(&mut stream, black_box(""))
                .expect("Failed to execute via daemon socket");
        });
    });

    stop_daemon();
}

// Configure Criterion with high sample size and measurement time for statistical stability
// Target: CV < 10% per AC6.4
// Daemon benchmarks should have lower variance than subprocess benchmarks due to:
// - No process spawn overhead
// - Warm cache reduces variance
// - Unix socket IPC is more consistent than process spawn
// Configuration:
// - sample_size: 1000 (increased for better statistical confidence)
// - measurement_time: 30s (sufficient time for 1000+ requests)
// - warm_up_time: 5s (daemon is pre-warmed manually)
// - noise_threshold: 0.05 (5% noise tolerance)
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(30))
        .warm_up_time(std::time::Duration::from_secs(5))
        .noise_threshold(0.05);
    targets =
        bench_daemon_mode_simple_arithmetic,
        bench_daemon_mode_complex_expression,
        bench_daemon_mode_with_variables,
        bench_daemon_mode_print,
        bench_daemon_mode_multiple_operations,
        bench_daemon_mode_by_complexity,
        bench_daemon_mode_cache_hit,
        bench_daemon_mode_throughput,
        bench_daemon_mode_minimal_overhead
}

criterion_main!(benches);
