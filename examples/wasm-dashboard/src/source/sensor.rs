// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{CancellationToken, FluxionTask};
use gloo_timers::future::TimeoutFuture;
use js_sys::Math;

pub struct Sensor {
    receiver: async_channel::Receiver<u32>,
    _task: FluxionTask,
}

impl Sensor {
    pub fn new(
        period_range_ms: (u64, u64),
        value_range: (u32, u32),
        cancel_token: CancellationToken,
    ) -> Self {
        let (sender, receiver) = async_channel::unbounded();
        let (min_period, max_period) = period_range_ms;
        let (min_value, max_value) = value_range;

        let task = FluxionTask::spawn(move |task_cancel| async move {
            loop {
                if cancel_token.is_cancelled() || task_cancel.is_cancelled() {
                    break;
                }

                let random_ratio = Math::random();
                let value_range = (max_value - min_value) as f64;
                let value = min_value + (random_ratio * value_range) as u32;

                if sender.send(value).await.is_err() {
                    break;
                }

                let random_delay = Math::random();
                let period_range = (max_period - min_period) as f64;
                let delay_ms = min_period + (random_delay * period_range) as u64;

                TimeoutFuture::new(delay_ms as u32).await;
            }
        });

        Self {
            receiver,
            _task: task,
        }
    }

    pub fn receiver(&self) -> async_channel::Receiver<u32> {
        self.receiver.clone()
    }
}
