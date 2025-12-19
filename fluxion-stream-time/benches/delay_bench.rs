// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use criterion::{BenchmarkId, Criterion, Throughput};
use fluxion_stream::IntoFluxionStream;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{TokioTimer, TokioTimestamped};
use futures::stream::StreamExt;
use std::hint::black_box;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::time::advance;

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
                        let timer = TokioTimer;
                        // 2. Create stream and operator
                        let (tx, rx) = mpsc::unbounded_channel();
                        let stream = rx.into_fluxion_stream().delay(duration);
                        let mut stream = Box::pin(stream);

                        // 3. Emit value
                        tx.send(TokioTimestamped::new(1, timer.now())).unwrap();

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
