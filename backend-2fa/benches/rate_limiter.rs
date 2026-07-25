use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petchain_2fa::rate_limiter::{InMemoryRateLimiter, RateLimiter};
use std::sync::Arc;
use std::thread;

fn bench_check_single_key(c: &mut Criterion) {
    let limiter = InMemoryRateLimiter::new(100, 60, 300);

    c.bench_function("record_failure_single_key", |b| {
        b.iter(|| limiter.record_failure("user:bench"))
    });
}

fn bench_check_many_keys(c: &mut Criterion) {
    let limiter = InMemoryRateLimiter::new(100, 60, 300);
    let keys: Vec<String> = (0..1000).map(|i| format!("user:{}", i)).collect();
    let mut idx = 0usize;

    c.bench_function("record_failure_rotating_keys", |b| {
        b.iter(|| {
            let result = limiter.record_failure(&keys[idx % keys.len()]);
            idx = idx.wrapping_add(1);
            result
        })
    });
}

fn bench_concurrent_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_record_failure");

    for threads in [2usize, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            &threads,
            |b, &n_threads| {
                b.iter(|| {
                    let limiter = Arc::new(InMemoryRateLimiter::new(1000, 60, 300));
                    let handles: Vec<_> = (0..n_threads)
                        .map(|t| {
                            let lim = Arc::clone(&limiter);
                            thread::spawn(move || {
                                for i in 0..100u32 {
                                    lim.record_failure(&format!("user:{}:{}", t, i));
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    rate_limiter_benches,
    bench_check_single_key,
    bench_check_many_keys,
    bench_concurrent_load,
);
criterion_main!(rate_limiter_benches);
