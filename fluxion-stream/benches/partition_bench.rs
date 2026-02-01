// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_stream::{IntoFluxionStream, PartitionExt};
use fluxion_test_utils::sequenced::Sequenced;
use futures::{future::join, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

pub fn bench_partition_balanced(c: &mut Criterion) {
    let mut group = c.benchmark_group("partition_balanced");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 64usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{size}_p{payload_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    let setup = || {
                        let (tx, rx) = unbounded::<Sequenced<Vec<u8>>>();
                        (tx, rx)
                    };

                    bencher.iter_with_setup(setup, |(tx, rx)| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let (true_stream, false_stream) =
                                rx.into_fluxion_stream().partition(|data: &Vec<u8>| {
                                    if data.is_empty() {
                                        true
                                    } else {
                                        data[0].is_multiple_of(2)
                                    }
                                });

                            for i in 0..size {
                                let _ = tx.try_send(Sequenced::new(vec![i as u8; payload_size]));
                            }
                            drop(tx);

                            let consume_true = async {
                                let mut s = Box::pin(true_stream);
                                while let Some(v) = s.next().await {
                                    black_box(v);
                                }
                            };
                            let consume_false = async {
                                let mut s = Box::pin(false_stream);
                                while let Some(v) = s.next().await {
                                    black_box(v);
                                }
                            };

                            join(consume_true, consume_false).await;
                        });
                    });
                },
            );
        }
    }

    group.finish();
}

pub fn bench_partition_imbalanced(c: &mut Criterion) {
    let mut group = c.benchmark_group("partition_imbalanced");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 64usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{size}_p{payload_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    let setup = || {
                        let (tx, rx) = unbounded::<Sequenced<Vec<u8>>>();
                        (tx, rx)
                    };

                    bencher.iter_with_setup(setup, |(tx, rx)| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let threshold = (size * 9 / 10) as u8;
                            let (true_stream, false_stream) =
                                rx.into_fluxion_stream().partition(move |data: &Vec<u8>| {
                                    if data.is_empty() {
                                        true
                                    } else {
                                        data[0] < threshold
                                    }
                                });

                            for i in 0..size {
                                let _ = tx.try_send(Sequenced::new(vec![i as u8; payload_size]));
                            }
                            drop(tx);

                            let consume_true = async {
                                let mut s = Box::pin(true_stream);
                                while let Some(v) = s.next().await {
                                    black_box(v);
                                }
                            };
                            let consume_false = async {
                                let mut s = Box::pin(false_stream);
                                while let Some(v) = s.next().await {
                                    black_box(v);
                                }
                            };

                            join(consume_true, consume_false).await;
                        });
                    });
                },
            );
        }
    }

    group.finish();
}

pub fn bench_partition_single_consumer(c: &mut Criterion) {
    let mut group = c.benchmark_group("partition_single_consumer");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 64usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{size}_p{payload_size}"));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    let setup = || {
                        let (tx, rx) = unbounded::<Sequenced<Vec<u8>>>();
                        (tx, rx)
                    };

                    bencher.iter_with_setup(setup, |(tx, rx)| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let (true_stream, _false_stream) =
                                rx.into_fluxion_stream().partition(|data: &Vec<u8>| {
                                    if data.is_empty() {
                                        true
                                    } else {
                                        data[0].is_multiple_of(2)
                                    }
                                });

                            for i in 0..size {
                                let _ = tx.try_send(Sequenced::new(vec![i as u8; payload_size]));
                            }
                            drop(tx);

                            let mut s = Box::pin(true_stream);
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
