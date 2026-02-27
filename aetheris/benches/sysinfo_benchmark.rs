use criterion::{criterion_group, criterion_main, Criterion};
use sysinfo::{System, SystemExt};

fn benchmark_sysinfo_refresh(c: &mut Criterion) {
    let mut sys = System::new_all();
    sys.refresh_all(); // Initial refresh to populate data

    c.bench_function("sysinfo_refresh_cpu_mem_disk", |b| {
        b.iter(|| {
            sys.refresh_cpu();
            sys.refresh_memory();
            sys.refresh_disks();
        })
    });
}

criterion_group!(benches, benchmark_sysinfo_refresh);
criterion_main!(benches);
