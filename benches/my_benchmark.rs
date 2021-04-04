use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rs3cache::definitions::location_configs::LocationConfig;

fn spam() {
    let configs = LocationConfig::dump_all().unwrap();
    black_box(configs);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("spam", |b| b.iter(|| spam()));
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(100);
    targets = criterion_benchmark
}
criterion_main!(benches);
