// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::TapExt;
use fluxion_test_utils::Sequenced;
use futures::stream::{self, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream(
    size: usize,
    payload_size: usize,
) -> impl futures::Stream<Item = StreamItem<Sequenced<Vec<u8>>>> {
    let items: Vec<Sequenced<Vec<u8>>> = (0..size)
        .map(|i| Sequenced::new(vec![i as u8; payload_size]))
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

/// Benchmarks tap with a minimal side effect (black_box to prevent optimization).
pub fn bench_tap(c: &mut Criterion) {
    let mut group = c.benchmark_group("tap");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{size}_p{payload_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    bencher.iter(|| {
                        let stream = make_stream(size, payload_size);

                        // Tap with minimal side effect - just observe values
                        let tapped = stream.tap(|data: &Vec<u8>| {
                            black_box(data.len());
                        });

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(tapped);
                            while let Some(v) = s.next().await {
                                black_box(v);
                            }
                        });
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmarks tap with multiple chained taps (common debugging pattern).
pub fn bench_tap_chained(c: &mut Criterion) {
    let mut group = c.benchmark_group("tap_chained");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{size}_p{payload_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    bencher.iter(|| {
                        let stream = make_stream(size, payload_size);

                        // Multiple taps in sequence (common for debugging pipelines)
                        let tapped = stream
                            .tap(|data: &Vec<u8>| {
                                black_box(data.len());
                            })
                            .tap(|data: &Vec<u8>| {
                                black_box(data.is_empty());
                            })
                            .tap(|data: &Vec<u8>| {
                                black_box(data.first());
                            });

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(tapped);
                            while let Some(v) = s.next().await {
                                black_box(v);
                            }
                        });
                    });
                },
            );
        }
    }

    group.finish();
}
