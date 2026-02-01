// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::aggregate::sensor_aggregate::SensorAggregate;
use crate::types::{humidity::Humidity, pressure::Pressure, temperature::Temperature};
use crate::{info, warn};
use core::time::Duration;
use fluxion_core::{CancellationToken, StreamItem};
use fluxion_stream::{
    DistinctUntilChangedByExt, DistinctUntilChangedExt, FilterOrderedExt, IntoFluxionStream,
    MergedStream, SkipItemsExt, TapExt, WindowByCountExt,
};
use fluxion_stream_time::EmbassyTimestamped;
use fluxion_stream_time::{DebounceExt, SampleExt, ThrottleExt};
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

    let temp_stream = temp_rx
        .into_fluxion_stream()
        .tap(|t| info!("Raw temperature: {:?} C", t.value_kelvin))
        .distinct_until_changed()
        .debounce(Duration::from_millis(500));
    info!("Temperature: tap -> distinct_until_changed -> debounce(500ms)");

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
        .filter_ordered(|p: &Pressure| p.value_hpa > 1000)
        .throttle(Duration::from_millis(750));
    info!("Pressure: distinct_until_changed_by -> filter_ordered(>1000hPa) -> throttle(750ms)");

    let humidity_stream = humidity_rx
        .into_fluxion_stream()
        .window_by_count::<EmbassyTimestamped<Vec<Humidity>>>(2)
        .skip_items(1)
        .sample(Duration::from_millis(100));
    info!("Humidity: window_by_count(2) -> skip_items(1) -> sample(100ms)");

    info!("\nMerging streams with stateful aggregation...\n");

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

    let mut merged = merged;
    loop {
        if _cancel.is_cancelled() {
            info!("\nFusion task cancelled");
            break;
        }

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
            Err(_) => continue,
        }
    }
}
