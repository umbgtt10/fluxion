use criterion::{BenchmarkId, Criterion, Throughput};
use futures::stream::{self, StreamExt};
use tokio::runtime::Runtime;
use fluxion_stream::timestamped::Timestamped;
use criterion::black_box;

// Small helper to create a stream of N Timestamped items with payload of given size
fn make_stream(n: usize, payload_size: usize) -> impl futures::Stream<Item = Timestamped<Vec<u8>>> {
    let items: Vec<Timestamped<Vec<u8>>> = (0..n)
        .map(|_i| Timestamped::new(vec![0u8; payload_size]))
        .collect();
    stream::iter(items)
}

// Placeholder: run merge_with benchmark
pub fn bench_merge_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_with");
    let items = [1000usize, 10_000usize];
    let payloads = [0usize, 128usize];

    for &m in &items {
        for &p in &payloads {
            let id = BenchmarkId::from_parameter(format!("merge_m{}_p{}", m, p));
            group.throughput(Throughput::Elements((m * 2) as u64));
            group.bench_with_input(id, &(m, p), |b, &(m, p)| {
                b.iter(|| {
                    let mut s1 = make_stream(m, p);
                    let mut s2 = make_stream(m, p);
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        while let Some(_v) = futures::stream::select(s1, s2).next().await {
                            black_box(())
                        }
                    });
                })
            });
        }
    }

    group.finish();
}
