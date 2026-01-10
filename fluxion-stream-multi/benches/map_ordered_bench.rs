// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream_multi::MapOrderedExt;
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

pub fn bench_map_ordered(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_ordered");
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
                    let setup = || make_stream(size, payload_size);

                    bencher.iter_with_setup(setup, |stream| {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            // Simple transformation: compute length of payload
                            let mapped = stream.map_ordered(|sequenced: Sequenced<Vec<u8>>| {
                                Sequenced::new(sequenced.value.len())
                            });
                            let mut s = Box::pin(mapped);
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
