// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::subject_bench::bench_subject;
use criterion::{criterion_group, criterion_main};

mod subject_bench;

criterion_group!(benches, bench_subject);
criterion_main!(benches);
