mod combine_latest;
mod merge_with;

use combine_latest::bench_combine_latest;
use merge_with::bench_merge_with;
use criterion::{criterion_group, criterion_main};

criterion_group!(combine_benches, bench_combine_latest);
criterion_group!(merge_benches, bench_merge_with);
criterion_main!(combine_benches, merge_benches);
