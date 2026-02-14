use criterion::{criterion_group, criterion_main, Criterion, black_box};
use aetheris_core::core::hardware::{HardwareInfo, HardwareProfile};
use aetheris_core::core::secrets::Secrets;
use aetheris_core::core::config::Config;
use aetheris_core::build_compose_structure;

fn benchmark_build_compose(c: &mut Criterion) {
    let hw = HardwareInfo {
        profile: HardwareProfile::Standard,
        ram_gb: 8,
        cpu_cores: 4,
        has_nvidia: false,
        has_intel_quicksync: false,
        disk_gb: 512,
        swap_gb: 2,
        user_id: "1000".to_string(),
        group_id: "1000".to_string(),
    };
    let secrets = Secrets::default();
    let config = Config::default();

    c.bench_function("build_compose_structure", |b| {
        b.iter(|| {
            let res = build_compose_structure(black_box(&hw), black_box(&secrets), black_box(&config)).unwrap();
            black_box(res);
        })
    });
}

criterion_group!(benches, benchmark_build_compose);
criterion_main!(benches);
