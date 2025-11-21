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
pub fn bench_take_while_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("take_while_with");
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
                        let source = make_stream(size, payload_size);
                        let filter = make_stream(size, payload_size);

                        let taken = source.take_while_with(filter, |_v| true);

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(taken);
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
