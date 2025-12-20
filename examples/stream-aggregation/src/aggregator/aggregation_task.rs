// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Aggregator task - combines three data sources using FluxionStream

use super::event_aggregation::create_aggregated_event;
use crate::consumer::FinalConsumer;
use crate::domain::{AggregatedEvent, DataEvent, MetricData, SensorReading, SystemEvent};
use crate::events_producer::EventsProducer;
use crate::metrics_producer::MetricsProducer;
use crate::sensor_producer::SensorProducer;
use fluxion_core::CancellationToken;
use fluxion_exec::SubscribeLatestExt;
use fluxion_rx::prelude::*;
use futures::channel::mpsc;
use std::convert::Infallible;
use tokio::task::JoinHandle;

pub struct Aggregator {
    cancel_token: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
    sensor_producer: Option<SensorProducer>,
    metrics_producer: Option<MetricsProducer>,
    events_producer: Option<EventsProducer>,
    consumer: Option<FinalConsumer>,
}

impl Aggregator {
    /// Creates a new aggregator with all producers and consumer
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token: cancel_token.clone(),
            task_handle: None,
            sensor_producer: Some(SensorProducer::new(cancel_token.clone())),
            metrics_producer: Some(MetricsProducer::new(cancel_token.clone())),
            events_producer: Some(EventsProducer::new(cancel_token.clone())),
            consumer: Some(FinalConsumer::new(cancel_token)),
        }
    }

    /// Starts all producers, aggregator task, and consumer
    pub fn start(&mut self) {
        // Create channels (simulating RabbitMQ queues)
        // Producers send raw data, aggregator handles sequencing
        let (sensor_tx, sensor_rx) = mpsc::unbounded();
        let (metrics_tx, metrics_rx) = mpsc::unbounded();
        let (events_tx, events_rx) = mpsc::unbounded();
        let (output_tx, output_rx) = mpsc::unbounded();

        // Start all tasks concurrently
        if let Some(producer) = &mut self.sensor_producer {
            producer.start(sensor_tx);
        }
        if let Some(producer) = &mut self.metrics_producer {
            producer.start(metrics_tx);
        }
        if let Some(producer) = &mut self.events_producer {
            producer.start(events_tx);
        }
        if let Some(consumer) = &mut self.consumer {
            consumer.start(output_rx);
        }

        // Start aggregator task
        let cancel_token = self.cancel_token.clone();
        let handle = tokio::spawn(async move {
            Self::run(sensor_rx, metrics_rx, events_rx, output_tx, cancel_token).await;
        });
        self.task_handle = Some(handle);
    }

    /// Stops all tasks (producers, aggregator, consumer)
    pub async fn stop(&mut self) {
        // Cancel the token to stop all tasks
        self.cancel_token.cancel();

        // Stop all tasks concurrently
        let sensor_stop = async {
            if let Some(producer) = &mut self.sensor_producer {
                producer.stop().await;
            }
        };
        let metrics_stop = async {
            if let Some(producer) = &mut self.metrics_producer {
                producer.stop().await;
            }
        };
        let events_stop = async {
            if let Some(producer) = &mut self.events_producer {
                producer.stop().await;
            }
        };
        let aggregator_stop = async {
            if let Some(handle) = self.task_handle.take() {
                let _ = handle.await;
            }
        };
        let consumer_stop = async {
            if let Some(consumer) = &mut self.consumer {
                consumer.stop().await;
            }
        };

        // Wait for all tasks to complete
        tokio::join!(
            sensor_stop,
            metrics_stop,
            events_stop,
            aggregator_stop,
            consumer_stop
        );
    }

    async fn run(
        sensor_rx: mpsc::UnboundedReceiver<SensorReading>,
        metrics_rx: mpsc::UnboundedReceiver<MetricData>,
        events_rx: mpsc::UnboundedReceiver<SystemEvent>,
        output_tx: mpsc::UnboundedSender<AggregatedEvent>,
        cancel_token: CancellationToken,
    ) {
        println!("🔄 Aggregator started\n");

        // Transform domain events to DataEvent (boxing happens inside into_fluxion_stream)
        let sensor_stream = sensor_rx.into_fluxion_stream_map(|s| DataEvent::Sensor(s.clone()));
        let metrics_stream = metrics_rx.into_fluxion_stream_map(|m| DataEvent::Metric(m.clone()));
        let events_stream =
            events_rx.into_fluxion_stream_map(|e| DataEvent::SystemEvent(e.clone()));

        // Combine the unified DataEvent streams
        let _ = sensor_stream
            .combine_latest(vec![metrics_stream, events_stream], |_| true)
            .map_ordered(create_aggregated_event)
            .filter_ordered(|agg| {
                // Only process reasonable temperatures (150-300 represents 15.0-30.0°C)
                agg.temperature.is_some_and(|t| (150..=300).contains(&t))
            })
            .subscribe_latest(
                move |stream_item, _token| {
                    let agg = stream_item.unwrap();
                    let tx = output_tx.clone();
                    async move {
                        let temp_display = agg.temperature.map(|t| t as f64 / 10.0).unwrap_or(0.0);
                        let metric_display = agg.metric_value.map(|m| m as f64).unwrap_or(0.0);

                        println!(
                            "\n  [Aggregator] @ {}: Temp={:.1}°C, Metric={:.1}, Alert={}",
                            agg.timestamp, temp_display, metric_display, agg.has_alert
                        );
                        let _ = tx.unbounded_send(agg);
                        Ok::<(), Infallible>(())
                    }
                },
                None::<fn(Infallible)>,
                Some(cancel_token),
            )
            .await;

        println!("\n🔄 Aggregator stopped");
    }
}
