mod combine_latest_bench;
mod combine_with_previous_bench;
mod ordered_merge_bench;
mod take_latest_when_bench;
mod take_while_with_bench;
mod with_latest_from_bench;

use criterion::{criterion_group, criterion_main};
use combine_latest_bench::bench_combine_latest;
use combine_with_previous_bench::bench_combine_with_previous;
use ordered_merge_bench::bench_ordered_merge;
use take_latest_when_bench::bench_take_latest_when;
use take_while_with_bench::bench_take_while_with;
use with_latest_from_bench::bench_with_latest_from;

criterion_group!(
    stream_benches,
    bench_combine_latest,
    bench_combine_with_previous,
    bench_ordered_merge,
    bench_take_latest_when,
    bench_take_while_with,
    bench_with_latest_from
);
criterion_main!(stream_benches);
