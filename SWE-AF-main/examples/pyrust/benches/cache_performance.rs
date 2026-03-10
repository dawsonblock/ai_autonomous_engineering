use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pyrust::ast::{Expression, Program, Statement};
use pyrust::bytecode::Bytecode;
use pyrust::cache::CompilationCache;
use pyrust::compiler::compile;
use std::sync::Arc;

/// Helper to create a simple bytecode for testing
fn create_bytecode(value: i64) -> Bytecode {
    let program = Program {
        statements: vec![Statement::Expression {
            value: Expression::Integer(value),
        }],
    };
    compile(&program).unwrap()
}

/// Benchmark: Cache hit latency (AC3.1 - target <50μs)
fn bench_cache_hit_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_latency");

    // Warm up cache
    let mut cache = CompilationCache::new(1000);
    let code = "2 + 3";
    let bytecode = create_bytecode(5);
    cache.insert(code.to_string(), Arc::new(bytecode));

    group.bench_function("cache_hit_simple_expression", |b| {
        b.iter(|| {
            let result = cache.get(black_box(code));
            assert!(result.is_some());
        });
    });

    group.finish();
}

/// Benchmark: Cache miss latency (should be <5% overhead per AC3.3)
fn bench_cache_miss_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_miss_latency");

    let mut cache = CompilationCache::new(1000);
    let code = "2 + 3";

    group.bench_function("cache_miss_simple_expression", |b| {
        b.iter(|| {
            let result = cache.get(black_box(code));
            assert!(result.is_none());
        });
    });

    group.finish();
}

/// Benchmark: Cache hit rate with repeated requests (AC3.1 - ≥95%)
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");
    // Increase sample size and measurement time to reduce CV
    group.sample_size(200);
    group.measurement_time(std::time::Duration::from_secs(10));

    group.bench_function("100_identical_requests", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(10);
            let code = "42";
            let bytecode = create_bytecode(42);

            // First request is a miss
            let _ = cache.get(code);
            cache.insert(code.to_string(), Arc::new(bytecode));

            // Next 99 requests should all be hits
            for _ in 0..99 {
                let result = cache.get(black_box(code));
                assert!(result.is_some());
            }

            let stats = cache.stats();
            assert!(stats.hit_rate >= 0.95, "Hit rate {} < 95%", stats.hit_rate);
        });
    });

    group.finish();
}

/// Benchmark: LRU eviction performance
fn bench_lru_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_eviction");

    for capacity in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("evict_at_capacity", capacity),
            capacity,
            |b, &cap| {
                b.iter(|| {
                    let mut cache = CompilationCache::new(cap);

                    // Fill cache to capacity
                    for i in 0..cap {
                        let code = format!("x = {}", i);
                        let bytecode = create_bytecode(i as i64);
                        cache.insert(code, Arc::new(bytecode));
                    }

                    // Insert one more to trigger eviction
                    let code = format!("x = {}", cap);
                    let bytecode = create_bytecode(cap as i64);
                    cache.insert(code, Arc::new(bytecode));

                    assert_eq!(cache.stats().size, cap);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Hash computation overhead
fn bench_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_computation");

    let short_code = "42";
    let medium_code = "x = 10\ny = 20\nz = x + y\nprint(z)";
    let long_code = "x = 42\n".repeat(100);

    group.bench_function("hash_short_code", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(10);
            cache.get(black_box(short_code));
        });
    });

    group.bench_function("hash_medium_code", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(10);
            cache.get(black_box(medium_code));
        });
    });

    group.bench_function("hash_long_code", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(10);
            cache.get(black_box(&long_code));
        });
    });

    group.finish();
}

/// Benchmark: Insert operation performance
fn bench_insert_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_performance");

    group.bench_function("insert_into_empty_cache", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(1000);
            let code = "42";
            let bytecode = create_bytecode(42);
            cache.insert(black_box(code.to_string()), Arc::new(bytecode));
        });
    });

    group.bench_function("insert_into_half_full_cache", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(1000);

            // Fill cache to 50%
            for i in 0..500 {
                let code = format!("x = {}", i);
                let bytecode = create_bytecode(i);
                cache.insert(code, Arc::new(bytecode));
            }

            // Insert new entry
            let code = "y = 42";
            let bytecode = create_bytecode(42);
            cache.insert(black_box(code.to_string()), Arc::new(bytecode));
        });
    });

    group.finish();
}

/// Benchmark: Cache statistics computation
fn bench_stats_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats_computation");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("compute_stats", size), size, |b, &s| {
            let mut cache = CompilationCache::new(1000);

            // Fill cache
            for i in 0..s {
                let code = format!("x = {}", i);
                let bytecode = create_bytecode(i);
                cache.insert(code, Arc::new(bytecode));
            }

            // Generate some hits/misses
            for i in 0..s / 2 {
                let code = format!("x = {}", i);
                cache.get(&code);
            }

            b.iter(|| {
                let stats = cache.stats();
                black_box(stats);
            });
        });
    }

    group.finish();
}

/// Benchmark: Realistic workload simulation
fn bench_realistic_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workload");

    group.bench_function("mixed_hit_miss_pattern", |b| {
        b.iter(|| {
            let mut cache = CompilationCache::new(100);

            // Simulate realistic workload:
            // - 80% cache hits (repeated code)
            // - 20% cache misses (new code)

            // Insert 10 common code patterns
            for i in 0..10 {
                let code = format!("common_{}", i);
                let bytecode = create_bytecode(i);
                cache.insert(code, Arc::new(bytecode));
            }

            // Simulate 100 requests with 80/20 hit/miss ratio
            for i in 0..100 {
                if i % 5 == 0 {
                    // 20% misses - new code
                    let code = format!("unique_{}", i);
                    cache.get(black_box(&code));
                    let bytecode = create_bytecode(i);
                    cache.insert(code, Arc::new(bytecode));
                } else {
                    // 80% hits - common code
                    let code = format!("common_{}", i % 10);
                    cache.get(black_box(&code));
                }
            }

            let stats = cache.stats();
            // Should have high hit rate (close to 80%)
            assert!(stats.hit_rate > 0.7, "Hit rate too low: {}", stats.hit_rate);
        });
    });

    group.finish();
}

// Configure Criterion with sample_size(1000) and measurement_time(10s) to reduce CV below 10% threshold
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3))
        .noise_threshold(0.05);
    targets =
        bench_cache_hit_latency,
        bench_cache_miss_latency,
        bench_cache_hit_rate,
        bench_lru_eviction,
        bench_hash_computation,
        bench_insert_performance,
        bench_stats_computation,
        bench_realistic_workload
}

criterion_main!(benches);
