// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream_multi::SampleRatioExt;
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

/// Benchmarks sample_ratio with 50% sampling.
pub fn bench_sample_ratio_half(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_ratio_half");
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
                    let setup = || {
                        let stream = make_stream(size, payload_size);

                        // Sample 50% of items with fixed seed for reproducibility
                        stream.sample_ratio(0.5, 42)
                    };

                    bencher.iter_with_setup(setup, |sampled| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(sampled);
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

/// Benchmarks sample_ratio with 100% pass-through.
pub fn bench_sample_ratio_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_ratio_full");
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
                    let setup = || {
                        let stream = make_stream(size, payload_size);

                        // Sample 100% - all items pass through
                        stream.sample_ratio(1.0, 42)
                    };

                    bencher.iter_with_setup(setup, |sampled| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(sampled);
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

/// Benchmarks sample_ratio with 10% sampling (aggressive downsampling).
pub fn bench_sample_ratio_sparse(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample_ratio_sparse");
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
                    let setup = || {
                        let stream = make_stream(size, payload_size);

                        // Sample 10% - aggressive downsampling
                        stream.sample_ratio(0.1, 42)
                    };

                    bencher.iter_with_setup(setup, |sampled| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(sampled);
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
