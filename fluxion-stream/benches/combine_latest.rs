use criterion::{BenchmarkId, Criterion, Throughput};
use futures::stream::{self, StreamExt};
use tokio::runtime::Runtime;
use fluxion_stream::timestamped::Timestamped;
use criterion::black_box;
use std::pin::Pin;

// Small helper to create a stream of N Timestamped items with payload of given size
fn make_stream(n: usize, payload_size: usize) -> impl futures::Stream<Item = Timestamped<Vec<u8>>> {
    let items: Vec<Timestamped<Vec<u8>>> = (0..n)
        .map(|_i| Timestamped::new(vec![0u8; payload_size]))
        .collect();
    stream::iter(items)
}

// Placeholder: run combine_latest benchmark (synchronous wrapper around async operator)
pub fn bench_combine_latest(c: &mut Criterion) {
    let mut group = c.benchmark_group("combine_latest");
    let stream_counts = [2usize, 4];
    let items = [1000usize, 10_000usize];
    let payloads = [0usize, 128usize];

    for &n in &stream_counts {
        for &m in &items {
            for &p in &payloads {
                let id = BenchmarkId::from_parameter(format!("combine_n{}_m{}_p{}", n, m, p));
                group.throughput(Throughput::Elements((n * m) as u64));
                group.bench_with_input(id, &(n, m, p), |b, &(n, m, p)| {
                    b.iter(|| {
                        // create n streams and drain them with select_all as a placeholder
                        let mut streams: Vec<Pin<Box<dyn futures::Stream<Item = Timestamped<Vec<u8>>> + Send>>> = Vec::new();
                        for _ in 0..n {
                            streams.push(Box::pin(make_stream(m, p)) as Pin<Box<dyn futures::Stream<Item = Timestamped<Vec<u8>>> + Send>>);
                        }
                        let mut combined = futures::stream::select_all(streams);
                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            while let Some(_v) = combined.next().await {
                                black_box(())
                            }
                        });
                    })
                });
            }
        }
    }

    group.finish();
}
