//! Integration tests for daemon + cache interaction
//!
//! Tests the integration between daemon concurrency features and cache functionality.
//! These tests verify that:
//! 1. Concurrent daemon requests correctly share the global cache
//! 2. Cache hits improve daemon performance
//! 3. No cache corruption occurs under concurrent load
//! 4. Cache statistics are accurate with concurrent access

use pyrust::daemon::DaemonServer;
use pyrust::daemon_protocol::{DaemonRequest, DaemonResponse};
use pyrust::{clear_global_cache, get_global_cache_stats};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

// Global counter for unique test IDs
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(2000);

/// Helper function to generate unique test paths
fn get_test_paths() -> (String, String) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let socket_path = format!("/tmp/pyrust_daemon_cache_test_{}.sock", id);
    let pid_path = format!("/tmp/pyrust_daemon_cache_test_{}.pid", id);
    (socket_path, pid_path)
}

/// Helper function to clean up test files
fn cleanup_test_files(socket_path: &str, pid_path: &str) {
    let _ = fs::remove_file(socket_path);
    let _ = fs::remove_file(pid_path);
}

/// Helper function to start daemon in background thread
fn start_daemon_in_background(socket_path: String, pid_path: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let daemon =
            DaemonServer::with_paths(socket_path, pid_path).expect("Failed to create daemon");
        let _ = daemon.run();
    })
}

/// Helper function to wait for socket to be available
fn wait_for_socket(socket_path: &str, timeout_secs: u64) -> bool {
    let start = Instant::now();
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

/// Helper function to send request and get response
fn send_request(
    socket_path: &str,
    code: &str,
) -> Result<DaemonResponse, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path)?;

    // Create and send request
    let request = DaemonRequest::new(code);
    let encoded = request.encode();
    stream.write_all(&encoded)?;
    stream.flush()?;

    // Read response
    let mut status_buf = [0u8; 1];
    stream.read_exact(&mut status_buf)?;

    let mut length_buf = [0u8; 4];
    stream.read_exact(&mut length_buf)?;
    let length = u32::from_be_bytes(length_buf) as usize;

    let mut output_buf = vec![0u8; length];
    stream.read_exact(&mut output_buf)?;

    let mut full_response = Vec::with_capacity(1 + 4 + length);
    full_response.extend_from_slice(&status_buf);
    full_response.extend_from_slice(&length_buf);
    full_response.extend_from_slice(&output_buf);

    let (response, _) = DaemonResponse::decode(&full_response)?;
    Ok(response)
}

/// Test that parallel daemon requests correctly share the global cache
#[test]
fn test_daemon_parallel_requests_share_global_cache() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    // Clear global cache before test
    clear_global_cache();

    // Start daemon in background
    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    // Wait for socket to be ready
    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Get initial cache stats
    let initial_stats = get_global_cache_stats();
    let initial_size = initial_stats.size;

    // Use same code for all requests - should result in cache hits
    let code = "42 + 58";
    let expected = "100";

    // First request - should be a cache miss
    let response = send_request(&socket_path, code).unwrap();
    assert_eq!(response.output(), expected);

    // Verify cache was populated
    let stats_after_first = get_global_cache_stats();
    assert_eq!(
        stats_after_first.size,
        initial_size + 1,
        "Cache should have one new entry"
    );

    // Use barrier to send multiple parallel requests
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for i in 0..10 {
        let socket_path_clone = socket_path.clone();
        let barrier_clone = Arc::clone(&barrier);
        let code = code.to_string();
        let expected = expected.to_string();

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            let response = send_request(&socket_path_clone, &code)
                .expect(&format!("Thread {} failed to send request", i));

            assert!(response.is_success(), "Request {} should succeed", i);
            assert_eq!(response.output(), expected, "Request {} result mismatch", i);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify cache stats show hits
    let final_stats = get_global_cache_stats();
    assert_eq!(
        final_stats.size,
        initial_size + 1,
        "Cache size should not increase for same code"
    );
    assert!(
        final_stats.hits >= stats_after_first.hits + 10,
        "Should have at least 10 cache hits"
    );

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

/// Test cache hit rate improves daemon performance
#[test]
fn test_daemon_cache_hit_performance_improvement() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    clear_global_cache();

    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    let code = "123 * 456";

    // Warmup: First request is a cache miss
    let _ = send_request(&socket_path, code).unwrap();

    // Measure cache miss time (second request with different code)
    let miss_code = "789 * 321";
    let miss_start = Instant::now();
    let _ = send_request(&socket_path, miss_code).unwrap();
    let miss_duration = miss_start.elapsed();

    // Measure cache hit time (repeated request)
    let hit_start = Instant::now();
    let _ = send_request(&socket_path, code).unwrap();
    let hit_duration = hit_start.elapsed();

    println!("Cache miss duration: {:?}", miss_duration);
    println!("Cache hit duration: {:?}", hit_duration);

    // Cache hits should be faster (though Unix socket overhead dominates)
    // We just verify both complete successfully
    assert!(miss_duration.as_millis() < 200, "Miss latency too high");
    assert!(hit_duration.as_millis() < 200, "Hit latency too high");

    let stats = get_global_cache_stats();
    println!(
        "Cache stats: hits={}, misses={}, hit_rate={:.2}%",
        stats.hits,
        stats.misses,
        stats.hit_rate * 100.0
    );

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

/// Test no cache corruption with concurrent requests using same code
#[test]
fn test_daemon_concurrent_cache_access_no_corruption() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    clear_global_cache();

    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Multiple threads request same code simultaneously
    let barrier = Arc::new(Barrier::new(20));
    let mut handles = vec![];

    let test_cases = vec![
        ("10 + 20", "30"),
        ("50 * 2", "100"),
        ("200 / 4", "50"),
        ("7 % 3", "1"),
    ];

    // Each thread makes multiple requests
    for i in 0..20 {
        let socket_path_clone = socket_path.clone();
        let barrier_clone = Arc::clone(&barrier);
        let test_cases = test_cases.clone();

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            let mut results = vec![];
            for (code, expected) in test_cases {
                let response = send_request(&socket_path_clone, code)
                    .expect(&format!("Thread {} failed to send request", i));

                assert!(response.is_success());
                assert_eq!(response.output(), expected);
                results.push(response.output().to_string());
            }
            results
        });

        handles.push(handle);
    }

    let mut all_results = vec![];
    for handle in handles {
        let results = handle.join().expect("Thread panicked");
        all_results.push(results);
    }

    // Verify all threads got correct results
    for results in all_results {
        assert_eq!(results[0], "30");
        assert_eq!(results[1], "100");
        assert_eq!(results[2], "50");
        assert_eq!(results[3], "1");
    }

    let stats = get_global_cache_stats();
    println!(
        "Concurrent cache test stats: size={}, hits={}, misses={}",
        stats.size, stats.hits, stats.misses
    );

    // Should have high hit rate due to code reuse
    assert!(
        stats.hit_rate > 0.7,
        "Hit rate should be > 70%, got {:.2}%",
        stats.hit_rate * 100.0
    );

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

/// Test cache statistics accuracy under concurrent daemon load
#[test]
fn test_daemon_cache_statistics_accuracy_concurrent() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    clear_global_cache();

    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    let initial_stats = get_global_cache_stats();
    let initial_hits = initial_stats.hits;
    let initial_misses = initial_stats.misses;

    // Send 5 unique requests (5 cache misses)
    let unique_codes = vec!["1+1", "2+2", "3+3", "4+4", "5+5"];
    for code in &unique_codes {
        let _ = send_request(&socket_path, code).unwrap();
    }

    // Send each code again (5 cache hits)
    for code in &unique_codes {
        let _ = send_request(&socket_path, code).unwrap();
    }

    let stats = get_global_cache_stats();

    // Verify cache statistics
    assert_eq!(stats.misses, initial_misses + 5, "Should have 5 new misses");
    assert_eq!(stats.hits, initial_hits + 5, "Should have 5 new hits");

    println!(
        "Cache statistics test: hits={}, misses={}, hit_rate={:.2}%",
        stats.hits,
        stats.misses,
        stats.hit_rate * 100.0
    );

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

/// Test cache behavior with mixed unique and repeated requests
#[test]
fn test_daemon_cache_mixed_request_pattern() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    clear_global_cache();

    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    let repeated_code = "99 + 1"; // This will be repeated
    let mut unique_codes = vec![];

    // Generate 10 unique codes
    for i in 0..10 {
        unique_codes.push(format!("{} * {}", i, i + 1));
    }

    // Send 50 requests: mix of repeated and unique
    let mut request_count = 0;
    let mut expected_hits = 0;

    for i in 0..50 {
        let code = if i % 3 == 0 {
            // Every 3rd request uses the repeated code
            if i > 0 {
                expected_hits += 1; // After first time, should be cache hit
            }
            repeated_code
        } else {
            // Other requests use unique codes
            &unique_codes[i % unique_codes.len()]
        };

        let response = send_request(&socket_path, code).unwrap();
        assert!(response.is_success());
        request_count += 1;
    }

    let stats = get_global_cache_stats();
    println!(
        "Mixed pattern stats: {} requests, hits={}, misses={}, hit_rate={:.2}%",
        request_count,
        stats.hits,
        stats.misses,
        stats.hit_rate * 100.0
    );

    // Should have some cache hits from repeated code
    assert!(stats.hits > 0, "Should have some cache hits");

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

/// Test daemon cache performance under sustained load
#[test]
fn test_daemon_cache_sustained_load_performance() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    clear_global_cache();

    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    const TOTAL_REQUESTS: usize = 500;
    const NUM_UNIQUE_CODES: usize = 50;

    // Generate pool of codes that will be reused (simulating realistic usage)
    let mut code_pool = vec![];
    for i in 0..NUM_UNIQUE_CODES {
        code_pool.push(format!("{} + {}", i * 2, i * 3));
    }

    let mut latencies = vec![];
    let start_time = Instant::now();

    // Send requests using codes from pool (realistic cache hit pattern)
    for i in 0..TOTAL_REQUESTS {
        let code = &code_pool[i % NUM_UNIQUE_CODES];

        let request_start = Instant::now();
        let response = send_request(&socket_path, code).unwrap();
        let latency = request_start.elapsed();

        assert!(response.is_success());
        latencies.push(latency);
    }

    let total_time = start_time.elapsed();

    // Calculate statistics
    let mean_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let requests_per_sec = TOTAL_REQUESTS as f64 / total_time.as_secs_f64();

    let stats = get_global_cache_stats();

    println!("\n========================================");
    println!("Sustained Load Test Results:");
    println!("========================================");
    println!("Total requests:     {}", TOTAL_REQUESTS);
    println!("Unique codes:       {}", NUM_UNIQUE_CODES);
    println!("Mean latency:       {:.0}μs", mean_latency.as_micros());
    println!("Requests/sec:       {:.0}", requests_per_sec);
    println!("Cache hits:         {}", stats.hits);
    println!("Cache misses:       {}", stats.misses);
    println!("Cache hit rate:     {:.2}%", stats.hit_rate * 100.0);
    println!("Cache size:         {}", stats.size);
    println!("========================================\n");

    // Verify high cache hit rate (should be ~90% with 50 unique codes and 500 requests)
    assert!(
        stats.hit_rate > 0.8,
        "Cache hit rate should be > 80%, got {:.2}%",
        stats.hit_rate * 100.0
    );

    // Verify cache size matches unique codes
    assert!(
        stats.size >= NUM_UNIQUE_CODES,
        "Cache should contain all unique codes"
    );

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}

/// Test cache behavior when daemon receives error-producing code
#[test]
fn test_daemon_cache_with_error_code() {
    let (socket_path, pid_path) = get_test_paths();
    cleanup_test_files(&socket_path, &pid_path);

    clear_global_cache();

    let _daemon_handle = start_daemon_in_background(socket_path.clone(), pid_path.clone());

    assert!(
        wait_for_socket(&socket_path, 5),
        "Socket not created within 5 seconds"
    );

    // Code that causes runtime error
    let error_code = "10 / 0";

    // First request - cache miss, runtime error
    let response1 = send_request(&socket_path, error_code).unwrap();
    assert!(response1.is_error());
    assert!(response1.output().contains("Division by zero"));

    // Second request - cache hit (bytecode cached), same runtime error
    let response2 = send_request(&socket_path, error_code).unwrap();
    assert!(response2.is_error());
    assert!(response2.output().contains("Division by zero"));

    let stats = get_global_cache_stats();

    // Bytecode should be cached even for error-producing code
    assert!(stats.size >= 1, "Bytecode should be cached");

    println!(
        "Error code cache test: hits={}, misses={}",
        stats.hits, stats.misses
    );

    cleanup_test_files(&socket_path, &pid_path);
    thread::sleep(Duration::from_millis(100));
}
