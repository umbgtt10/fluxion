// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use futures::stream::StreamExt;
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::time::advance;

/// # Panics
///
/// This benchmark constructs a local `Runtime` with `Runtime::new().unwrap()`, which may panic.
pub fn bench_delay(c: &mut Criterion) {
    let mut group = c.benchmark_group("delay_overhead");
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
                        let (tx, rx) = mpsc::unbounded_channel();
                        let stream = FluxionStream::from_unbounded_receiver(rx).delay(duration);
                        let mut stream = Box::pin(stream);

                        // 3. Emit value
                        tx.send(ChronoTimestamped::now(1)).unwrap();

                        // 4. Advance time instantly (0 wall-clock time, pure CPU cost)
                        advance(duration).await;

                        // 5. Assert result
                        let item = stream.next().await;
                        black_box(item);
                    });
                });
            },
        );
    }

    group.finish();
}
