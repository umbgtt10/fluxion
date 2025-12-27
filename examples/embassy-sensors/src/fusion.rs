// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor fusion task - combines all three sensor streams.

use crate::types::{Humidity, Pressure, SensorReading, Temperature};
use core::time::Duration;
use fluxion_core::{CancellationToken, StreamItem};
use fluxion_stream::IntoFluxionStream;
use fluxion_stream_time::prelude::*;
use futures::StreamExt;

/// Sensor fusion task - combines all three streams with reactive pipelines
#[embassy_executor::task]
pub async fn fusion_task(
    temp_rx: async_channel::Receiver<Temperature>,
    pressure_rx: async_channel::Receiver<Pressure>,
    humidity_rx: async_channel::Receiver<Humidity>,
    _cancel: CancellationToken,
) {
    println!("🔄 Fusion task started - demonstrating time operators");
    println!("Building reactive pipeline with combine_latest...\n");

    // Use into_fluxion_stream_map to transform AND timestamp in one step
    let temp_stream = temp_rx
        .into_fluxion_stream_map(|t| SensorReading::Temperature(t.clone()))
        .debounce(Duration::from_millis(100));
    println!("✓ Temperature: mapped -> debounce(100ms)");

    let pressure_stream = pressure_rx
        .into_fluxion_stream_map(|p| SensorReading::Pressure(p.clone()))
        .throttle(Duration::from_millis(200));
    println!("✓ Pressure: mapped -> throttle(200ms)");

    let humidity_stream = humidity_rx
        .into_fluxion_stream_map(|h| SensorReading::Humidity(h.clone()))
        .sample(Duration::from_millis(150));
    println!("✓ Humidity: mapped -> sample(150ms)");

    let mut combined = temp_stream.combine_latest([pressure_stream, humidity_stream]);
    println!("✓ Combined all streams with combine_latest");
    println!("\n📡 Waiting for sensor readings...\n");

    // Process combined stream
    let mut count = 0;
    loop {
        // Check cancellation
        if _cancel.is_cancelled() {
            println!("\n🔄 Fusion task cancelled after {} readings", count);
            break;
        }

        // Poll with timeout for cancellation responsiveness
        match embassy_time::with_timeout(embassy_time::Duration::from_millis(500), combined.next())
            .await
        {
            Ok(Some(items)) => {
                count += 1;
                println!("📦 Combined reading #{}:", count);

                for item in items {
                    if let StreamItem::Value(val) = item {
                        match val {
                            SensorReading::Temperature(t) => {
                                println!("   └─ 🌡️  Temperature: {:.2}°C", t.value as f32 / 100.0)
                            }
                            SensorReading::Pressure(p) => {
                                println!("   └─ 📊 Pressure: {:.2} hPa", p.value as f32 / 100.0)
                            }
                            SensorReading::Humidity(h) => {
                                println!("   └─ 💧 Humidity: {:.2}%", h.value as f32 / 100.0)
                            }
                        }
                    }
                }
                println!();
            }
            Ok(None) => {
                println!("\n🔄 Stream ended");
                break;
            }
            Err(_) => continue, // Timeout, check cancellation
        }
    }
}
