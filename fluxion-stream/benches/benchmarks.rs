mod merge_with;

use criterion::{criterion_group, criterion_main};
use merge_with::bench_merge_with;

criterion_group!(merge_benches, bench_merge_with);
criterion_main!(merge_benches);
