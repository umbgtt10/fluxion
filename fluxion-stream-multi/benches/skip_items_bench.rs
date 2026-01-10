// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_stream_multi::SkipItemsExt;
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

pub fn bench_skip_items(c: &mut Criterion) {
    let mut group = c.benchmark_group("skip_items");
    let sizes = [100usize, 1000usize, 10000];
    let payload_sizes = [16usize, 32usize, 64usize, 128usize];
    let skip_percentages = [10usize, 50usize, 90usize]; // Percentage of stream to skip

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            for &skip_pct in &skip_percentages {
                let skip_count = (size * skip_pct) / 100;
                let emit_count = size - skip_count;
                let id =
                    BenchmarkId::from_parameter(format!("m{size}_p{payload_size}_s{skip_pct}pct"));
                // Throughput is items emitted after skip, not total stream size
                group.throughput(Throughput::Elements(emit_count as u64));
                group.bench_with_input(
                    id,
                    &(size, payload_size, skip_count),
                    |bencher, &(size, payload_size, skip_count)| {
                        let setup = || make_stream(size, payload_size);

                        bencher.iter_with_setup(setup, |stream| {
                            let rt = Runtime::new().unwrap();
                            rt.block_on(async move {
                                let skipped = stream.skip_items(skip_count);
                                let mut s = Box::pin(skipped);
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
