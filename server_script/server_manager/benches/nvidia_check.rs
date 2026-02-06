use criterion::{criterion_group, criterion_main, Criterion};
use which::which;

fn check_nvidia_eager() -> bool {
    let has_smi = which("nvidia-smi").is_ok();
    let has_cli = which("nvidia-container-cli").is_ok();
    let has_runtime = which("nvidia-container-runtime").is_ok();

    has_smi && (has_cli || has_runtime)
}

fn check_nvidia_lazy() -> bool {
    if which("nvidia-smi").is_err() {
        return false;
    }
    let has_cli = which("nvidia-container-cli").is_ok();
    let has_runtime = which("nvidia-container-runtime").is_ok();

    has_cli || has_runtime
}

fn benchmark_nvidia_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("nvidia_check");

    group.bench_function("eager", |b| {
        b.iter(|| {
            check_nvidia_eager()
        })
    });

    group.bench_function("lazy", |b| {
        b.iter(|| {
            check_nvidia_lazy()
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_nvidia_checks);
criterion_main!(benches);
