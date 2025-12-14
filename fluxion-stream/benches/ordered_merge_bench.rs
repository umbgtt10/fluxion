// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::Sequenced;
use futures::stream::{self, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream(
    size: usize,
    payload_size: usize,
) -> FluxionStream<impl futures::Stream<Item = StreamItem<Sequenced<Vec<u8>>>>> {
    let items: Vec<Sequenced<Vec<u8>>> = (0..size)
        .map(|_i| Sequenced::new(vec![0u8; payload_size]))
        .collect();
    FluxionStream::new(stream::iter(items).map(StreamItem::Value))
}

/// # Panics
///
/// This benchmark constructs a local `Runtime` with `Runtime::new().unwrap()`, which may panic.
pub fn bench_ordered_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordered_merge");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];
    let num_streams_variants = [2usize, 3usize, 5usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            for &num_streams in &num_streams_variants {
                let id =
                    BenchmarkId::from_parameter(format!("m{size}_p{payload_size}_s{num_streams}"));
                group.throughput(Throughput::Elements((size * num_streams) as u64));
                group.bench_with_input(
                    id,
                    &(size, payload_size, num_streams),
                    |bencher, &(size, payload_size, num_streams)| {
                        bencher.iter(|| {
                            let first_stream = make_stream(size, payload_size);
                            let other_streams: Vec<_> = (1..num_streams)
                                .map(|_| make_stream(size, payload_size))
                                .collect();

                            let merged = first_stream.ordered_merge(other_streams);

                            let rt = Runtime::new().unwrap();
                            rt.block_on(async move {
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
    }

    group.finish();
}
