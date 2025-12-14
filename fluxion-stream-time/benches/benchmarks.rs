// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{criterion_group, criterion_main};

mod debounce_bench;
mod delay_bench;
mod sample_bench;
mod throttle_bench;
mod timeout_bench;

use debounce_bench::bench_debounce;
use delay_bench::bench_delay;
use sample_bench::bench_sample;
use throttle_bench::bench_throttle;
use timeout_bench::bench_timeout;

criterion_group!(
    benches,
    bench_debounce,
    bench_delay,
    bench_sample,
    bench_throttle,
    bench_timeout
);
criterion_main!(benches);
