// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_core::StreamItem;
use fluxion_core::{FluxionSubject, HasTimestamp, Timestamped};
use futures::StreamExt;
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::runtime::Runtime;

// Simple timestamped wrapper for benchmarks (no test-utils dependency)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BenchValue<T> {
    value: T,
    timestamp: u64,
}

impl<T> BenchValue<T> {
    fn new(value: T) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self {
            value,
            timestamp: COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl<T> HasTimestamp for BenchValue<T> {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T: Clone> Timestamped for BenchValue<T> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.value
    }

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { value, timestamp }
    }
}

pub fn bench_subject(c: &mut Criterion) {
    let mut group = c.benchmark_group("subject");

    // Subscriber counts to test scalability
    let subscriber_counts = [1usize, 8, 64, 256];

    // Scenario 1: small numeric payload (BenchValue<u64>)
    for &subs in &subscriber_counts {
        group.throughput(Throughput::Elements(subs as u64));
        let id = BenchmarkId::from_parameter(format!("simple_subs_{subs}"));
        group.bench_with_input(id, &subs, |bencher, &subs| {
            // Setup: create subject and spawn subscribers (not timed)
            let setup = || {
                let rt = Runtime::new().unwrap();
                let subj: Arc<FluxionSubject<BenchValue<u64>>> = Arc::new(FluxionSubject::new());

                // Spawn subscriber tasks that await a single item
                let mut handles = Vec::with_capacity(subs);
                for _ in 0..subs {
                    let s = subj.subscribe();
                    handles.push(tokio::spawn(async move {
                        let mut s = s.unwrap();
                        let item = s.next().await;
                        black_box(item);
                    }));
                }
                (rt, subj, handles)
            };

            bencher.iter_with_setup(setup, |(rt, subj, handles)| {
                // Only measure send/wait time
                rt.block_on(async {
                    // Send a small numeric value (no test fixtures used)
                    subj.send(StreamItem::Value(BenchValue::new(42u64)))
                        .unwrap();

                    // Wait for subscribers
                    for h in handles {
                        let _ = h.await;
                    }
                });
            });
        });
    }

    // Scenario 2: large payload cloning cost - use Vec<u8>
    let payload_sizes = [256usize, 1024usize, 4096usize];
    for &size in &payload_sizes {
        for &subs in &subscriber_counts {
            group.throughput(Throughput::Bytes((size * subs) as u64));
            let id = BenchmarkId::from_parameter(format!("large_p{}_subs_{}", size, subs));
            group.bench_with_input(id, &(size, subs), |bencher, &(size, subs)| {
                // Setup: create subject and spawn subscribers (not timed)
                let setup = || {
                    let rt = Runtime::new().unwrap();
                    let subj: Arc<FluxionSubject<BenchValue<Vec<u8>>>> =
                        Arc::new(FluxionSubject::new());

                    let mut handles = Vec::with_capacity(subs);
                    for _ in 0..subs {
                        let s = subj.subscribe();
                        handles.push(tokio::spawn(async move {
                            let mut s = s.unwrap();
                            let item = s.next().await;
                            black_box(item);
                        }));
                    }
                    (rt, subj, handles)
                };

                bencher.iter_with_setup(setup, |(rt, subj, handles)| {
                    // Only measure send/wait time
                    rt.block_on(async {
                        let payload = vec![0u8; size];
                        subj.send(StreamItem::Value(BenchValue::new(payload)))
                            .unwrap();

                        for h in handles {
                            let _ = h.await;
                        }
                    });
                });
            });
        }
    }

    group.finish();
}
