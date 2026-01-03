// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::merge_with::MergedStream;
use fluxion_test_utils::Sequenced;
use futures::{
    stream::{self, StreamExt},
    Stream,
};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream(
    size: usize,
    payload_size: usize,
) -> impl Stream<Item = StreamItem<Sequenced<Vec<u8>>>> {
    let items: Vec<StreamItem<Sequenced<Vec<u8>>>> = (0..size)
        .map(|_i| Sequenced::new(vec![0u8; payload_size]))
        .map(StreamItem::Value)
        .collect();
    stream::iter(items)
}

pub fn bench_merge_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_with");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{size}_p{payload_size}"));
            group.throughput(Throughput::Elements((size * 3) as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    let setup = || {
                        let stream1 = make_stream(size, payload_size);
                        let stream2 = make_stream(size, payload_size);
                        let stream3 = make_stream(size, payload_size);
                        (stream1, stream2, stream3)
                    };

                    bencher.iter_with_setup(setup, |(stream1, stream2, stream3)| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            // Repository pattern: merge streams with state counter
                            let merged = MergedStream::seed::<Sequenced<usize>>(0)
                                .merge_with(stream1, |_item: Vec<u8>, state: &mut usize| {
                                    *state += 1;
                                    *state
                                })
                                .merge_with(stream2, |_item: Vec<u8>, state: &mut usize| {
                                    *state += 1;
                                    *state
                                })
                                .merge_with(stream3, |_item: Vec<u8>, state: &mut usize| {
                                    *state += 1;
                                    *state
                                });
                            let mut s = Box::pin(merged);
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
