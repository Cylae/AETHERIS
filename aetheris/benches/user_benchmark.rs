use criterion::{criterion_group, criterion_main, Criterion};
use aetheris_core::core::users::{UserManager, Role};
use aetheris_core::adapters::mock::MockRuntime;

fn benchmark_verify_async(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut manager = UserManager::default();
    let mock_runtime = MockRuntime::default();

    // Setup user
    runtime.block_on(async {
        manager.add_user(&mock_runtime, "benchuser", "password123", Role::Observer, None).await.unwrap();
    });

    let mut group = c.benchmark_group("user_verification");

    // Benchmark successful login
    group.bench_function("verify_async_success", |b| {
        b.to_async(&runtime).iter(|| async {
            manager.verify_async("benchuser", "password123").await
        })
    });

    // Benchmark failed login (wrong password)
    group.bench_function("verify_async_fail_password", |b| {
        b.to_async(&runtime).iter(|| async {
            manager.verify_async("benchuser", "wrongpassword").await
        })
    });

    // Benchmark failed login (wrong username) - this should be very fast as it returns None immediately
    group.bench_function("verify_async_fail_username", |b| {
        b.to_async(&runtime).iter(|| async {
            manager.verify_async("nonexistent", "password123").await
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_verify_async);
criterion_main!(benches);
