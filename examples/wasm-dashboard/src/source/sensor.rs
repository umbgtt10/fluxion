// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{CancellationToken, FluxionTask};
use gloo_timers::future::TimeoutFuture;
use js_sys::Math;

/// A single sensor that generates random values at random intervals.
///
/// The sensor spawns a background task that continuously generates values
/// within the specified range at frequencies between 1-5 Hz (200-1000ms intervals).
///
/// The task is automatically cancelled when the `Sensor` is dropped or when
/// the provided cancellation token is triggered.
pub struct Sensor {
    receiver: async_channel::Receiver<u32>,
    _task: FluxionTask,
}

impl Sensor {
    /// Creates a new sensor.
    ///
    /// # Arguments
    ///
    /// * `period_range_ms` - Min and max period in milliseconds (e.g., (200, 1000) for 1-5 Hz)
    /// * `value_range` - Min and max values to generate (inclusive)
    /// * `cancel_token` - External cancellation token to stop the sensor
    ///
    /// # Example
    ///
    /// ```ignore
    /// let cancel = CancellationToken::new();
    /// let sensor = Sensor::new((200, 1000), (1, 9), cancel.clone());
    /// ```
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
                // Check both external and task cancellation tokens
                if cancel_token.is_cancelled() || task_cancel.is_cancelled() {
                    break;
                }

                // Generate random value in range [min_value, max_value]
                let random_ratio = Math::random();
                let value_range = (max_value - min_value) as f64;
                let value = min_value + (random_ratio * value_range) as u32;

                // Send value (ignore errors if receiver dropped)
                if sender.send(value).await.is_err() {
                    break;
                }

                // Generate random delay in range [min_period, max_period]
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

    /// Returns a cloneable receiver for this sensor's data.
    ///
    /// Multiple receivers can be created from the same sensor stream,
    /// each receiving all values emitted by the sensor.
    pub fn receiver(&self) -> async_channel::Receiver<u32> {
        self.receiver.clone()
    }
}
