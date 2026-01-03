// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_stream::{IntoFluxionStream, ShareExt};
use fluxion_test_utils::Sequenced;
use futures::{future::join_all, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

pub fn bench_share(c: &mut Criterion) {
    let mut group = c.benchmark_group("share");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];
    let subscriber_counts = [1usize, 2usize, 4usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            for &subscribers in &subscriber_counts {
                let id =
                    BenchmarkId::from_parameter(format!("m{size}_p{payload_size}_s{subscribers}"));
                group.throughput(Throughput::Elements(size as u64));
                group.bench_with_input(
                    id,
                    &(size, payload_size, subscribers),
                    |bencher, &(size, payload_size, subscribers)| {
                        let setup = || {
                            let (tx, rx) = unbounded::<Sequenced<Vec<u8>>>();
                            (tx, rx)
                        };

                        bencher.iter_with_setup(setup, |(tx, rx)| {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async move {
                                let stream = rx.into_fluxion_stream();
                                let shared = stream.share();

                                // Create subscriber futures BEFORE sending data
                                let futures: Vec<_> = (0..subscribers)
                                    .map(|_| {
                                        let sub = shared.subscribe().unwrap();
                                        async move {
                                            let mut s = Box::pin(sub);
                                            while let Some(v) = s.next().await {
                                                black_box(v);
                                            }
                                        }
                                    })
                                    .collect();

                                // Now send data through the channel
                                for _ in 0..size {
                                    let _ = tx.try_send(Sequenced::new(vec![0u8; payload_size]));
                                }
                                drop(tx); // Close channel to signal completion

                                // Drive all subscribers to completion without spawning tasks
                                join_all(futures).await;
                            });
                        });
                    },
                );
            }
        }
    }

    group.finish();
}
