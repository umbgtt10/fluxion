// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

mod ordered_merge_bench;
mod select_all_bench;

use criterion::{criterion_group, criterion_main};
use ordered_merge_bench::bench_ordered_merge;
use select_all_bench::bench_select_all;

criterion_group!(merge_benches, bench_ordered_merge, bench_select_all);
criterion_main!(merge_benches);
