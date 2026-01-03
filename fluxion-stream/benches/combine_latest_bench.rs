// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::CombineLatestExt;
use fluxion_test_utils::Sequenced;
use futures::stream::{self, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream(
    size: usize,
    payload_size: usize,
) -> impl futures::Stream<Item = StreamItem<Sequenced<Vec<u8>>>> {
    let items: Vec<Sequenced<Vec<u8>>> = (0..size)
        .map(|_i| Sequenced::new(vec![0u8; payload_size]))
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

pub fn bench_combine_latest(c: &mut Criterion) {
    let mut group = c.benchmark_group("combine_latest");
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
                    // Setup: create streams (not timed)
                    let setup = || {
                        let stream1 = make_stream(size, payload_size);
                        let stream2 = make_stream(size, payload_size);
                        let stream3 = make_stream(size, payload_size);
                        (stream1, stream2, stream3)
                    };

                    bencher.iter_with_setup(setup, |(stream1, stream2, stream3)| {
                        // Only measure execution time
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let combined =
                                stream1.combine_latest(vec![stream2, stream3], |_state| true);
                            let mut s = Box::pin(combined);
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
