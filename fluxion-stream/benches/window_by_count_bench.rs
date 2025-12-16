// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::WindowByCountExt;
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
        .map(|i| Sequenced::new(vec![i as u8; payload_size]))
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

/// Benchmarks window_by_count with various window sizes.
pub fn bench_window_by_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_by_count");
    let sizes = [100usize, 1000usize, 10000];
    let window_sizes = [2usize, 5usize, 10usize, 50usize];

    for &size in &sizes {
        for &window_size in &window_sizes {
            let id = BenchmarkId::from_parameter(format!("n{size}_w{window_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(id, &(size, window_size), |bencher, &(size, window_size)| {
                bencher.iter(|| {
                    let stream = make_stream(size);

                    let windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(window_size);

                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        let mut s = Box::pin(windowed);
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

/// Benchmarks window_by_count with various payload sizes.
pub fn bench_window_by_count_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_by_count_payload");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 64usize, 256usize];
    let window_size = 10usize;

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("n{size}_p{payload_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    bencher.iter(|| {
                        let stream = make_stream_with_payload(size, payload_size);

                        let windowed =
                            stream.window_by_count::<Sequenced<Vec<Vec<u8>>>>(window_size);

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(windowed);
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

/// Benchmarks window_by_count with window size 1 (each item becomes its own window).
pub fn bench_window_by_count_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_by_count_single");
    let sizes = [100usize, 1000usize, 10000];

    for &size in &sizes {
        let id = BenchmarkId::from_parameter(format!("n{size}"));
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(id, &size, |bencher, &size| {
            bencher.iter(|| {
                let stream = make_stream(size);

                // Window size 1 = maximum number of windows
                let windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(1);

                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let mut s = Box::pin(windowed);
                    while let Some(v) = s.next().await {
                        black_box(v);
                    }
                });
            });
        });
    }

    group.finish();
}
