use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use std::fs;
use tokio::fs as async_fs;

fn benchmark_io_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_dir = std::env::temp_dir();
    let blocking_file = temp_dir.join("blocking_test.txt");
    let async_file = temp_dir.join("async_test.txt");
    let content = "vm.swappiness=10\nfs.inotify.max_user_watches=524288\n";

    let mut group = c.benchmark_group("io_operations");

    group.bench_function("blocking_fs_write", |b| {
        b.iter(|| {
            fs::write(&blocking_file, content).unwrap();
        })
    });

    // For criterion 0.5, to_async requires the "async_tokio" feature
    // which might not be enabled. We can simulate it or just use block_on.
    group.bench_function("async_fs_write", |b| {
        b.iter(|| {
             rt.block_on(async {
                 async_fs::write(&async_file, content).await.unwrap();
             })
        })
    });

    group.finish();

    // Cleanup
    let _ = fs::remove_file(blocking_file);
    let _ = fs::remove_file(async_file);
}

criterion_group!(benches, benchmark_io_operations);
criterion_main!(benches);
