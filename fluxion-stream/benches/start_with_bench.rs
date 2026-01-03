// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::StartWithExt;
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
    let items: Vec<Sequenced<Vec<u8>>> = (0..size)
        .map(|i| Sequenced::new(vec![i as u8; payload_size]))
        .collect();
    stream::iter(items).map(StreamItem::Value)
}

pub fn bench_start_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("start_with");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];
    let initial_counts = [0usize, 10usize, 100usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            for &initial_count in &initial_counts {
                let id = BenchmarkId::from_parameter(format!(
                    "m{size}_p{payload_size}_i{initial_count}"
                ));
                // Throughput is total elements (initial + stream)
                group.throughput(Throughput::Elements((size + initial_count) as u64));
                group.bench_with_input(
                    id,
                    &(size, payload_size, initial_count),
                    |bencher, &(size, payload_size, initial_count)| {
                        let setup = || make_stream(size, payload_size);

                        bencher.iter_with_setup(setup, |stream| {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async move {
                                // Create initial values
                                let initial: Vec<StreamItem<Sequenced<Vec<u8>>>> = (0
                                    ..initial_count)
                                    .map(|i| {
                                        StreamItem::Value(Sequenced::new(vec![
                                            i as u8;
                                            payload_size
                                        ]))
                                    })
                                    .collect();

                                let with_initial = stream.start_with(initial);
                                let mut s = Box::pin(with_initial);
                                while let Some(v) = s.next().await {
                                    black_box(v);
                                }
                            });
                        });
                    },
                );
            }
        }
    }

    group.finish();
}
