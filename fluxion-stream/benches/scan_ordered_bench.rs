// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::ScanOrderedExt;
use fluxion_test_utils::Sequenced;
use futures::{
    stream::{self, StreamExt},
    Stream,
};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream(size: usize) -> impl Stream<Item = StreamItem<Sequenced<i32>>> {
    let items: Vec<Sequenced<i32>> = (1..=size as i32).map(Sequenced::new).collect();
    stream::iter(items).map(StreamItem::Value)
}

fn make_stream_with_payload(
    size: usize,
    payload_size: usize,
) -> impl Stream<Item = StreamItem<Sequenced<Vec<u8>>>> {
    let items: Vec<Sequenced<Vec<u8>>> = (0..size)
        .map(|_i| Sequenced::new(vec![0u8; payload_size]))
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

pub fn bench_scan_ordered_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_ordered_sum");
    let sizes = [100usize, 1000usize, 10000];

    for &size in &sizes {
        let id = BenchmarkId::from_parameter(format!("n{size}"));
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(id, &size, |bencher, &size| {
            bencher.iter(|| {
                let stream = make_stream(size);

                // Running sum accumulator
                let scanned =
                    stream.scan_ordered::<Sequenced<i32>, _, _>(0, |sum: &mut i32, value: &i32| {
                        *sum += value;
                        *sum
                    });

                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let mut s = Box::pin(scanned);
                    while let Some(v) = s.next().await {
                        black_box(v);
                    }
                });
            });
        });
    }

    group.finish();
}

pub fn bench_scan_ordered_vec_accumulator(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_ordered_vec_accumulator");
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
                        let stream = make_stream_with_payload(size, payload_size);

                        // Accumulate lengths into a vector
                        let scanned = stream.scan_ordered::<Sequenced<Vec<usize>>, _, _>(
                            Vec::new(),
                            |lengths: &mut Vec<usize>, payload: &Vec<u8>| {
                                lengths.push(payload.len());
                                lengths.clone() // Return snapshot
                            },
                        );

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(scanned);
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

pub fn bench_scan_ordered_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_ordered_count");
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
                        let stream = make_stream_with_payload(size, payload_size);

                        // Simple counter
                        let scanned = stream.scan_ordered::<Sequenced<i32>, _, _>(
                            0,
                            |count: &mut i32, _: &Vec<u8>| {
                                *count += 1;
                                *count
                            },
                        );

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(scanned);
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
