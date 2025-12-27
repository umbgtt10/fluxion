// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::processing::CombinedStream;
use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream_time::DebounceWithDefaultTimerExt;
use fluxion_stream_time::DelayWithDefaultTimerExt;
use fluxion_stream_time::SampleWithDefaultTimerExt;
use fluxion_stream_time::ThrottleWithDefaultTimerExt;
use fluxion_stream_time::TimeoutWithDefaultTimerExt;
use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::time::Duration;

/// WASM-compatible stream type (without Send+Sync bounds required for threading)
pub type WasmStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>>>>;

/// Factory for creating time-operator streams from the combined stream.
///
/// In WASM, timer-based streams cannot be shared (multi-subscribed) because
/// they contain !Send JavaScript callbacks. This struct creates fresh streams
/// on-demand instead of storing shared instances.
pub struct ResultStreams<'a> {
    combined_stream: &'a CombinedStream,
}

impl<'a> ResultStreams<'a> {
    /// Creates a new factory for time-operator streams.
    ///
    /// # Arguments
    ///
    /// * `combined_stream` - The combined stream to apply operators to
    pub fn new(combined_stream: &'a CombinedStream) -> Self {
        Self { combined_stream }
    }

    /// Creates a new debounced stream (700ms).
    ///
    /// Note: Each call creates a new stream subscription. Not shareable in WASM.
    pub fn subscribe_debounce(&self) -> WasmStream<u32> {
        Box::pin(
            self.combined_stream
                .subscribe()
                .debounce(Duration::from_millis(700))
                .map(|item| item.map(|timestamped| timestamped.into_inner())),
        )
    }

    /// Creates a new delayed stream (1000ms).
    ///
    /// Note: Each call creates a new stream subscription. Not shareable in WASM.
    pub fn subscribe_delay(&self) -> WasmStream<u32> {
        Box::pin(
            self.combined_stream
                .subscribe()
                .delay(Duration::from_millis(1000))
                .map(|item| item.map(|timestamped| timestamped.into_inner())),
        )
    }

    /// Creates a new sampled stream (1000ms).
    ///
    /// Note: Each call creates a new stream subscription. Not shareable in WASM.
    pub fn subscribe_sample(&self) -> WasmStream<u32> {
        Box::pin(
            self.combined_stream
                .subscribe()
                .sample(Duration::from_millis(1000))
                .map(|item| item.map(|timestamped| timestamped.into_inner())),
        )
    }

    /// Creates a new throttled stream (800ms).
    ///
    /// Note: Each call creates a new stream subscription. Not shareable in WASM.
    pub fn subscribe_throttle(&self) -> WasmStream<u32> {
        Box::pin(
            self.combined_stream
                .subscribe()
                .throttle(Duration::from_millis(800))
                .map(|item| item.map(|timestamped| timestamped.into_inner())),
        )
    }

    /// Creates a new stream with timeout (2000ms).
    ///
    /// Note: Each call creates a new stream subscription. Not shareable in WASM.
    pub fn subscribe_timeout(&self) -> WasmStream<u32> {
        Box::pin(
            self.combined_stream
                .subscribe()
                .timeout(Duration::from_millis(2000))
                .map(|item| item.map(|timestamped| timestamped.into_inner())),
        )
    }
}
