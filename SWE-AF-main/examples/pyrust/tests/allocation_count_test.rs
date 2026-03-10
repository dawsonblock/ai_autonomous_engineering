//! Allocation profiling tests using dhat
//!
//! These tests measure exact allocation counts for execute_python calls to validate
//! AC2 (Memory Efficiency): Total allocations ≤ 5 per execute_python("2 + 3") call.
//!
//! Run with: cargo test --features dhat-heap test_allocation_count -- --ignored --nocapture --test-threads=1
//!
//! Note: --test-threads=1 is required because dhat can only have one profiler instance at a time

#![cfg(not(miri))] // Disable under Miri (doesn't support dhat)

use pyrust::execute_python;

/// Test allocation count for simple expression (2 + 3)
/// AC2: Total allocations ≤ 5 per execute_python("2 + 3") call
#[test]
#[ignore] // Run with: cargo test test_allocation_count -- --ignored
#[cfg(not(miri))]
fn test_allocation_count() {
    // Initialize dhat profiler
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Warm up (ensures JIT/caching doesn't affect measurement)
    // 100 iterations to populate caches and stabilize allocation patterns
    for _ in 0..100 {
        let _ = execute_python("2 + 3");
    }

    // Measure allocations for single execution
    #[cfg(feature = "dhat-heap")]
    let stats_before = dhat::HeapStats::get();

    let result = execute_python("2 + 3").unwrap();
    assert_eq!(result, "5");

    #[cfg(feature = "dhat-heap")]
    let stats_after = dhat::HeapStats::get();

    // Calculate allocation count
    #[cfg(feature = "dhat-heap")]
    {
        let alloc_count = stats_after.total_blocks - stats_before.total_blocks;
        eprintln!(
            "Allocation count for execute_python(\"2 + 3\"): {}",
            alloc_count
        );

        // AC2: Total allocations ≤ 5
        assert!(
            alloc_count <= 5,
            "Allocation count {} exceeds target of 5",
            alloc_count
        );
    }

    #[cfg(not(feature = "dhat-heap"))]
    {
        // When not profiling, test still validates correctness
        println!("Note: Run with --features dhat-heap to measure allocations");
    }
}

/// Test allocation count for program with variables
/// Variables program may have slightly higher allocation budget (≤8)
#[test]
#[ignore] // Run with: cargo test test_allocation_count_with_variables -- --ignored
#[cfg(not(miri))]
fn test_allocation_count_with_variables() {
    // Initialize dhat profiler
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // Warm up - 100 iterations to populate caches
    for _ in 0..100 {
        let _ = execute_python("x = 10\ny = 20\nx + y");
    }

    #[cfg(feature = "dhat-heap")]
    let stats_before = dhat::HeapStats::get();

    let result = execute_python("x = 10\ny = 20\nx + y").unwrap();
    assert_eq!(result, "30");

    #[cfg(feature = "dhat-heap")]
    let stats_after = dhat::HeapStats::get();

    #[cfg(feature = "dhat-heap")]
    {
        let alloc_count = stats_after.total_blocks - stats_before.total_blocks;
        eprintln!("Allocation count for variables program: {}", alloc_count);

        // Variables program may have slightly higher allocation budget
        assert!(
            alloc_count <= 8,
            "Allocation count {} exceeds target of 8 for variables",
            alloc_count
        );
    }

    #[cfg(not(feature = "dhat-heap"))]
    {
        // When not profiling, test still validates correctness
        println!("Note: Run with --features dhat-heap to measure allocations");
    }
}
