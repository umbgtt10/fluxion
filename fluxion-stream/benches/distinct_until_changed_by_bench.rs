// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::DistinctUntilChangedByExt;
use fluxion_test_utils::Sequenced;
use futures::{
    stream::{self, StreamExt},
    Stream,
};
use std::hint::black_box;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
struct Record {
    id: u32,
    value: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedF64(f64);

impl Eq for OrderedF64 {}

impl PartialOrd for OrderedF64 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn make_stream_with_duplicates(
    size: usize,
    duplicate_factor: usize,
) -> impl Stream<Item = StreamItem<Sequenced<Record>>> {
    // Create stream with consecutive duplicates based on id
    let items: Vec<Sequenced<Record>> = (0..size)
        .map(|i| {
            Sequenced::new(Record {
                id: (i / duplicate_factor) as u32,
                value: format!("value_{}", i),
            })
        })
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

fn make_stream_case_insensitive(size: usize) -> impl Stream<Item = StreamItem<Sequenced<String>>> {
    // Create stream with case variations
    let items: Vec<Sequenced<String>> = (0..size)
        .map(|i| {
            let base = format!("item_{}", i / 2);
            if i % 2 == 0 {
                Sequenced::new(base.to_lowercase())
            } else {
                Sequenced::new(base.to_uppercase())
            }
        })
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

fn make_stream_threshold(size: usize) -> impl Stream<Item = StreamItem<Sequenced<OrderedF64>>> {
    // Create stream with values that vary within/outside threshold
    let items: Vec<Sequenced<OrderedF64>> = (0..size)
        .map(|i| {
            let base = (i / 10) as f64;
            let variation = (i % 10) as f64 * 0.01; // Small variations
            Sequenced::new(OrderedF64(base + variation))
        })
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

pub fn bench_distinct_until_changed_by(c: &mut Criterion) {
    // Benchmark field comparison
    let mut group = c.benchmark_group("distinct_until_changed_by_field");
    let sizes = [100usize, 1000usize, 10000];
    let duplicate_factors = [1usize, 2usize, 5usize, 10usize];

    for &size in &sizes {
        for &dup_factor in &duplicate_factors {
            let id = BenchmarkId::from_parameter(format!("m{size}_dup{dup_factor}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(id, &(size, dup_factor), |bencher, &(size, dup_factor)| {
                let setup = || {
                    let stream = make_stream_with_duplicates(size, dup_factor);
                    stream.distinct_until_changed_by(|a, b| a.id == b.id)
                };

                bencher.iter_with_setup(setup, |distinct| {
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

    // Benchmark case-insensitive comparison
    let mut group = c.benchmark_group("distinct_until_changed_by_case_insensitive");
    let sizes = [100usize, 1000usize, 10000];

    for &size in &sizes {
        let id = BenchmarkId::from_parameter(format!("m{size}"));
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(id, &size, |bencher, &size| {
            let setup = || {
                let stream = make_stream_case_insensitive(size);
                stream.distinct_until_changed_by(|a, b| a.to_lowercase() == b.to_lowercase())
            };

            bencher.iter_with_setup(setup, |distinct| {
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

    // Benchmark threshold comparison (floating point)
    let mut group = c.benchmark_group("distinct_until_changed_by_threshold");
    let sizes = [100usize, 1000usize, 10000];
    let thresholds = [0.05f64, 0.1f64, 0.5f64];

    for &size in &sizes {
        for &threshold in &thresholds {
            let id = BenchmarkId::from_parameter(format!("m{size}_t{}", threshold));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(id, &(size, threshold), |bencher, &(size, threshold)| {
                let setup = || {
                    let stream = make_stream_threshold(size);
                    stream.distinct_until_changed_by(move |a, b| (a.0 - b.0).abs() < threshold)
                };

                bencher.iter_with_setup(setup, |distinct| {
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
}
