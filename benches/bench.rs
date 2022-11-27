use std::collections::BTreeMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rs3cache::{cli::Config, definitions::item_configs::ItemConfig};
#[inline]
fn create_config() -> BTreeMap<u32, ItemConfig> {
    let config = Config::env();
    black_box(ItemConfig::dump_all(&config).unwrap())
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("create_config", |b| b.iter(create_config));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
