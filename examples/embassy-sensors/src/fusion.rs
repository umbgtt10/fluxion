// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor fusion task - combines all three sensor streams.

use crate::aggregate::SensorAggregate;
use crate::types::{Humidity, Pressure, Temperature};
use crate::{info, warn};
use core::time::Duration;
use fluxion_core::{CancellationToken, StreamItem};
use fluxion_stream::{
    DistinctUntilChangedByExt, DistinctUntilChangedExt, FilterOrderedExt, IntoFluxionStream,
    MergedStream, SkipItemsExt, TapExt, WindowByCountExt,
};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::EmbassyTimestamped;
use futures::StreamExt;

extern crate alloc;
use alloc::vec::Vec;

#[embassy_executor::task]
pub async fn fusion_task(
    temp_rx: async_channel::Receiver<Temperature>,
    pressure_rx: async_channel::Receiver<Pressure>,
    humidity_rx: async_channel::Receiver<Humidity>,
    _cancel: CancellationToken,
) {
    info!("Fusion task started - demonstrating operators and merge_with");
    info!("Building reactive pipeline with multiple operators...\n");

    // Temperature stream: tap -> distinct_until_changed -> debounce
    let temp_stream = temp_rx
        .into_fluxion_stream()
        .tap(|t| info!("Raw temperature: {:?} C", t.value_kelvin))
        .distinct_until_changed()
        .debounce(Duration::from_millis(500));
    info!("Temperature: tap -> distinct_until_changed -> debounce(500ms)");

    // Pressure stream: distinct_until_changed_by -> filter_ordered -> throttle
    let pressure_stream = pressure_rx
        .into_fluxion_stream()
        .distinct_until_changed_by(|p1, p2| {
            let difference = p1.value_hpa as i32 - p2.value_hpa as i32;
            info!(
                "Pressure change: {} hPa (from {} to {})",
                difference, p2.value_hpa, p1.value_hpa
            );
            difference.abs() < 10
        })
        .filter_ordered(|p: &Pressure| p.value_hpa > 1000) // Filter by pressure value
        .throttle(Duration::from_millis(750));
    info!("Pressure: distinct_until_changed_by -> filter_ordered(>1000hPa) -> throttle(750ms)");

    // Humidity stream: window_by_count -> skip_items -> sample
    let humidity_stream = humidity_rx
        .into_fluxion_stream()
        .window_by_count::<EmbassyTimestamped<Vec<Humidity>>>(2) // Batch into pairs
        .skip_items(1) // Skip first window (only 1 item)
        .sample(Duration::from_millis(100));
    info!("Humidity: window_by_count(2) -> skip_items(1) -> sample(100ms)");

    info!("\nMerging streams with stateful aggregation...\n");

    // Merge streams with shared state accumulation
    let merged = MergedStream::seed::<EmbassyTimestamped<SensorAggregate>>(SensorAggregate::new())
        .merge_with(temp_stream, |temp: Temperature, state| {
            state.latest_temp = Some(temp);
            state.update_count += 1;
            state.clone()
        })
        .merge_with(
            pressure_stream,
            |pressure: Pressure, state: &mut SensorAggregate| {
                state.latest_pressure = Some(pressure);
                state.update_count += 1;
                state.clone()
            },
        )
        .merge_with(
            humidity_stream,
            |humidity_window: Vec<Humidity>, state: &mut SensorAggregate| {
                if humidity_window.len() == 2 {
                    let prev = &humidity_window[0];
                    let curr = &humidity_window[1];
                    state.humidity_delta = curr.value_percent as i32 - prev.value_percent as i32;
                    state.latest_humidity = Some(*curr);
                }
                state.update_count += 1;
                state.clone()
            },
        );

    // Process merged stream with manual loop
    let mut merged = merged;
    loop {
        // Check cancellation
        if _cancel.is_cancelled() {
            info!("\nFusion task cancelled");
            break;
        }

        // Poll with timeout for cancellation responsiveness
        match embassy_time::with_timeout(embassy_time::Duration::from_millis(500), merged.next())
            .await
        {
            Ok(Some(StreamItem::Value(aggregate))) => {
                if aggregate.value.is_complete() {
                    info!("Complete sensor aggregate received: {}", aggregate.value);
                }
            }
            Ok(Some(StreamItem::Error(e))) => {
                warn!("Error: {}", e);
            }
            Ok(None) => {
                info!("\nStream ended");
                break;
            }
            Err(_) => continue, // Timeout, check cancellation
        }
    }
}
