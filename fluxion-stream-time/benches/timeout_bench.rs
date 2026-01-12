// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream::IntoFluxionStream;
use fluxion_stream_time::TimeoutExt;
use fluxion_stream_time::TokioTimestamped;
use futures::StreamExt;
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time::advance;

pub fn bench_timeout(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeout_overhead");
    let durations = [Duration::from_millis(10), Duration::from_secs(1)];

    for &duration in &durations {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", duration)),
            &duration,
            |bencher, &duration| {
                bencher.iter(|| {
                    // 1. Setup a lightweight, paused runtime
                    let rt = Builder::new_current_thread()
                        .enable_time()
                        .start_paused(true)
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        // 2. Create stream and operator
                        let (tx, rx) = async_channel::unbounded();
                        let stream = rx.into_fluxion_stream().timeout(duration);
                        let mut stream = Box::pin(stream);

                        // 3. Emit value BEFORE timeout
                        tx.try_send(TokioTimestamped::new(1, TokioTimer.now()))
                            .unwrap();

                        // 4. Assert result (should be immediate)
                        let item = stream.next().await;
                        black_box(item);

                        // 5. Advance time to ensure timer cleanup doesn't crash
                        advance(duration).await;
                    });
                });
            },
        );
    }

    group.finish();
}
