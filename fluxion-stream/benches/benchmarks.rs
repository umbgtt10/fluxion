// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod combine_latest_bench;
mod combine_with_previous_bench;
mod distinct_until_changed_bench;
mod distinct_until_changed_by_bench;
mod emit_when_bench;
mod filter_ordered_bench;
mod map_ordered_bench;
mod merge_with_bench;
mod ordered_merge_bench;
mod partition_bench;
mod sample_ratio_bench;
mod scan_ordered_bench;
mod share_bench;
mod skip_items_bench;
mod start_with_bench;
mod take_items_bench;
mod take_latest_when_bench;
mod take_while_with_bench;
mod tap_bench;
mod with_latest_from_bench;

use combine_latest_bench::bench_combine_latest;
use combine_with_previous_bench::bench_combine_with_previous;
use criterion::{criterion_group, criterion_main};
use distinct_until_changed_bench::bench_distinct_until_changed;
use distinct_until_changed_by_bench::bench_distinct_until_changed_by;
use emit_when_bench::bench_emit_when;
use filter_ordered_bench::bench_filter_ordered;
use map_ordered_bench::bench_map_ordered;
use merge_with_bench::bench_merge_with;
use ordered_merge_bench::bench_ordered_merge;
use partition_bench::{
    bench_partition_balanced, bench_partition_imbalanced, bench_partition_single_consumer,
};
use sample_ratio_bench::{
    bench_sample_ratio_full, bench_sample_ratio_half, bench_sample_ratio_sparse,
};
use scan_ordered_bench::{
    bench_scan_ordered_count, bench_scan_ordered_sum, bench_scan_ordered_vec_accumulator,
};
use share_bench::bench_share;
use skip_items_bench::bench_skip_items;
use start_with_bench::bench_start_with;
use take_items_bench::bench_take_items;
use take_latest_when_bench::bench_take_latest_when;
use take_while_with_bench::bench_take_while_with;
use tap_bench::{bench_tap, bench_tap_chained};
use with_latest_from_bench::bench_with_latest_from;

criterion_group!(
    stream_benches,
    bench_combine_latest,
    bench_combine_with_previous,
    bench_distinct_until_changed,
    bench_distinct_until_changed_by,
    bench_emit_when,
    bench_filter_ordered,
    bench_map_ordered,
    bench_merge_with,
    bench_ordered_merge,
    bench_partition_balanced,
    bench_partition_imbalanced,
    bench_partition_single_consumer,
    bench_sample_ratio_full,
    bench_sample_ratio_half,
    bench_sample_ratio_sparse,
    bench_scan_ordered_count,
    bench_scan_ordered_sum,
    bench_scan_ordered_vec_accumulator,
    bench_share,
    bench_skip_items,
    bench_start_with,
    bench_take_items,
    bench_take_latest_when,
    bench_take_while_with,
    bench_tap,
    bench_tap_chained,
    bench_with_latest_from
);
criterion_main!(stream_benches);
