// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod merge_ordered_bench;
mod select_all_bench;

use criterion::{criterion_group, criterion_main};
use merge_ordered_bench::bench_ordered_merge;
use select_all_bench::bench_select_all;

criterion_group!(merge_benches, bench_ordered_merge, bench_select_all);
criterion_main!(merge_benches);
