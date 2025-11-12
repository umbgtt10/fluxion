use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::stream::{self, StreamExt};
use tokio::runtime::Runtime;
use std::time::Duration;
use fluxion_stream::timestamped::Timestamped;

// Small helper to create a stream of N Timestamped items with payload of given size
fn make_stream(n: usize, payload_size: usize) -> impl futures::Stream<Item = Timestamped<Vec<u8>>> {
    let items: Vec<Timestamped<Vec<u8>>> = (0..n)
        .map(|i| Timestamped::new(vec![0u8; payload_size]))
        .collect();
    stream::iter(items)
}

// Placeholder: run combine_latest benchmark (synchronous wrapper around async operator)
fn run_combine_latest_benchmark(n_streams: usize, items_per_stream: usize, payload: usize) {
    // For now just create streams and iterate to drain; replace with real operator wiring
    let mut streams = Vec::new();
    for _ in 0..n_streams {
        streams.push(Box::pin(make_stream(items_per_stream, payload)) as Pin<Box<dyn futures::Stream<Item = Timestamped<Vec<u8>>> + Send>>);
    }
    // Simple drain
    let mut combined = futures::stream::select_all(streams);
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        while let Some(_v) = combined.next().await {
            // black_box? keep minimal
        }
    });
}

// Placeholder: run merge_with benchmark
fn run_merge_with_benchmark(items: usize, payload: usize) {
    let mut s1 = make_stream(items, payload);
    let mut s2 = make_stream(items, payload);
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        while let Some(_v) = futures::stream::select(s1, s2).next().await {}
    });
}

use std::pin::Pin;
use criterion::black_box;

fn bench_combine_and_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("combine_and_merge");
    let stream_counts = [2usize, 4];
    let items = [1000usize, 10_000usize];
    let payloads = [0usize, 128usize];

    for &n in &stream_counts {
        for &m in &items {
            for &p in &payloads {
                let id = BenchmarkId::from_parameter(format!("combine_n{}_m{}_p{}", n, m, p));
                group.throughput(Throughput::Elements((n * m) as u64));
                group.bench_with_input(id, &(n, m, p), |b, &(n, m, p)| {
                    b.iter(|| run_combine_latest_benchmark(black_box(n), black_box(m), black_box(p)))
                });

                let id2 = BenchmarkId::from_parameter(format!("merge_m{}_p{}", m, p));
                group.throughput(Throughput::Elements((m * 2) as u64));
                group.bench_with_input(id2, &(m, p), |b, &(m, p)| {
                    b.iter(|| run_merge_with_benchmark(black_box(m), black_box(p)))
                });
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_combine_and_merge);
criterion_main!(benches);
