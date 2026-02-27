use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use std::fs;
use std::sync::Arc;

fn benchmark_io(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = std::env::temp_dir().join("aetheris_bench");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).unwrap();
    }
    fs::create_dir_all(&temp_dir).unwrap();
    let temp_dir = Arc::new(temp_dir);

    let mut group = c.benchmark_group("io_operations");
    let task_count = 50; // Enough to saturate a few workers if blocking

    group.bench_function("blocking_fs_write", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();
            for i in 0..task_count {
                let dir = temp_dir.clone();
                handles.push(tokio::spawn(async move {
                    let path = dir.join(format!("blocking_{}.txt", i));
                    // Intentionally blocking the thread
                    std::fs::write(path, "content").unwrap();
                }));
            }
            for h in handles {
                h.await.unwrap();
            }
        })
    });

    group.bench_function("async_fs_write", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();
            for i in 0..task_count {
                let dir = temp_dir.clone();
                handles.push(tokio::spawn(async move {
                    let path = dir.join(format!("async_{}.txt", i));
                    tokio::fs::write(path, "content").await.unwrap();
                }));
            }
            for h in handles {
                h.await.unwrap();
            }
        })
    });

    group.finish();
    // Cleanup
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&*temp_dir);
    }
}

criterion_group!(benches, benchmark_io);
criterion_main!(benches);
