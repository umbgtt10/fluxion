use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::sequenced::Sequenced;
use futures::stream::{self, StreamExt};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn make_stream(
    size: usize,
    payload_size: usize,
) -> impl futures::Stream<Item = Sequenced<Vec<u8>>> {
    let items: Vec<Sequenced<Vec<u8>>> = (0..size)
        .map(|_i| Sequenced::new(vec![0u8; payload_size]))
        .collect();
    stream::iter(items)
}

pub fn bench_ordered_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordered_merge");
    let sizes = [100usize, 1000usize, 10_000usize];
    let payload_sizes = [0usize, 128usize];
    let num_streams_variants = [2usize, 3usize, 5usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            for &num_streams in &num_streams_variants {
                let id = BenchmarkId::from_parameter(format!(
                    "m{}_p{}_s{}",
                    size, payload_size, num_streams
                ));
                group.throughput(Throughput::Elements((size * num_streams) as u64));
                group.bench_with_input(
                    id,
                    &(size, payload_size, num_streams),
                    |bencher, &(size, payload_size, num_streams)| {
                        bencher.iter(|| {
                            let streams: Vec<_> = (0..num_streams)
                                .map(|_| make_stream(size, payload_size))
                                .collect();

                            let merged = streams.ordered_merge();

                            let rt = Runtime::new().unwrap();
                            rt.block_on(async move {
                                let mut s = Box::pin(merged);
                                while let Some(v) = s.next().await {
                                    black_box(v);
                                }
                            });
                        })
                    },
                );
            }
        }
    }

    group.finish();
}
