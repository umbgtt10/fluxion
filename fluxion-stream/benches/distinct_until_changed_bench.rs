// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::DistinctUntilChangedExt;
use fluxion_test_utils::Sequenced;
use futures::stream::{self, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream_with_duplicates(
    size: usize,
    duplicate_factor: usize,
) -> impl futures::Stream<Item = StreamItem<Sequenced<i32>>> {
    // Create stream with consecutive duplicates
    // Each unique value repeated duplicate_factor times
    let items: Vec<Sequenced<i32>> = (0..size)
        .flat_map(|i| {
            std::iter::repeat_n(
                Sequenced::new(i as i32 / duplicate_factor as i32),
                duplicate_factor,
            )
        })
        .take(size)
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

fn make_stream_alternating(size: usize) -> impl futures::Stream<Item = StreamItem<Sequenced<i32>>> {
    // Create stream with no consecutive duplicates (worst case - no filtering)
    let items: Vec<Sequenced<i32>> = (0..size).map(|i| Sequenced::new((i % 2) as i32)).collect();
    stream::iter(items).map(StreamItem::Value)
}

pub fn bench_distinct_until_changed(c: &mut Criterion) {
    let mut group = c.benchmark_group("distinct_until_changed");
    let sizes = [100usize, 1000usize, 10000];
    let duplicate_factors = [1usize, 2usize, 5usize, 10usize]; // 1 = no duplicates, 10 = 90% filtered

    for &size in &sizes {
        for &dup_factor in &duplicate_factors {
            let id = BenchmarkId::from_parameter(format!("m{size}_dup{dup_factor}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(id, &(size, dup_factor), |bencher, &(size, dup_factor)| {
                bencher.iter(|| {
                    let stream = make_stream_with_duplicates(size, dup_factor);
                    let distinct = stream.distinct_until_changed();

                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        let mut s = Box::pin(distinct);
                        while let Some(v) = s.next().await {
                            black_box(v);
                        }
                    });
                });
            });
        }
    }

    group.finish();

    // Benchmark worst case (alternating values - no filtering)
    let mut group = c.benchmark_group("distinct_until_changed_worst_case");
    let sizes = [100usize, 1000usize, 10000];

    for &size in &sizes {
        let id = BenchmarkId::from_parameter(format!("m{size}_alternating"));
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(id, &size, |bencher, &size| {
            bencher.iter(|| {
                let stream = make_stream_alternating(size);
                let distinct = stream.distinct_until_changed();

                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let mut s = Box::pin(distinct);
                    while let Some(v) = s.next().await {
                        black_box(v);
                    }
                });
            });
        });
    }

    group.finish();
}
