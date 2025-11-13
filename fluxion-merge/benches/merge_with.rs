use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_merge::MergedStream;
use fluxion_stream::sequenced::Sequenced;
use futures::stream::{self, StreamExt};
use std::hint::black_box;
use std::pin::Pin;
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

pub fn bench_merge_with(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_with");
    let sizes = [1000usize, 10_000usize];
    let payload_sizes = [0usize, 128usize];

    for &size in &sizes {
        for &payload_size in &payload_sizes {
            let id = BenchmarkId::from_parameter(format!("merge_m{}_p{}", size, payload_size));
            group.throughput(Throughput::Elements((size * 3) as u64));
            group.bench_with_input(
                id,
                &(size, payload_size),
                |bencher, &(size, payload_size)| {
                    bencher.iter(|| {
                        let new_stream1 = make_stream(size, payload_size);
                        let new_stream2 = make_stream(size, payload_size);
                        let new_stream3 = make_stream(size, payload_size);

                        #[derive(Default)]
                        struct SharedState {
                            processed: u64,
                        }

                        impl SharedState {
                            fn update1(&mut self) {
                                self.processed += 1;
                            }

                            fn update2(&mut self) {
                                self.processed += 2;
                            }

                            fn update3(&mut self) {
                                self.processed += 3;
                            }
                        }

                        let merged = MergedStream::seed(SharedState::default())
                            .merge_with(new_stream1, |new_item, state| {
                                state.update1();
                                Sequenced::new(new_item.into_inner())
                            })
                            .merge_with(new_stream2, |new_item, state| {
                                state.update2();
                                Sequenced::new(new_item.into_inner())
                            })
                            .merge_with(new_stream3, |new_item, state| {
                                state.update3();
                                Sequenced::new(new_item.into_inner())
                            });

                        let rt = Runtime::new().unwrap();
                        rt.block_on(async move {
                            let mut s = Box::pin(merged)
                                as Pin<Box<dyn futures::Stream<Item = Sequenced<Vec<u8>>> + Send>>;
                            while let Some(_v) = s.next().await {
                                black_box(())
                            }
                        });
                    })
                },
            );
        }
    }

    group.finish();
}
