// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::subject_bench::bench_subject;
use criterion::{criterion_group, criterion_main};

mod subject_bench;

criterion_group!(benches, bench_subject);
criterion_main!(benches);
