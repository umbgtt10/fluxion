use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_stream::WithLatestFromExt;
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

pub fn bench_with_latest_from(c: &mut Criterion) {
    let mut group = c.benchmark_group("with_latest_from");
    let sizes = [100usize, 1000usize, 10_000usize];
    let payload_sizes = [0usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("m{}_p{}", size, payload_size));
            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    bencher.iter(|| {
                        let primary = make_stream(size, payload_size);
                        let secondary = make_stream(size, payload_size);

                        let combined = primary.with_latest_from(secondary, |_state| true);

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(combined);
                            while let Some(v) = s.next().await {
                                black_box(v);
                            }
                        });
                    })
                },
            );
        }
    }

    group.finish();
}
