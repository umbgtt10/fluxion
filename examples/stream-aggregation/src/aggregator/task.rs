// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Aggregator task - combines three data sources using FluxionStream

use crate::consumer::FinalConsumer;
use crate::domain::{AggregatedEvent, DataEvent, MetricData, SensorReading, SystemEvent};
use crate::events_producer::EventsProducer;
use crate::metrics_producer::MetricsProducer;
use crate::sensor_producer::SensorProducer;
use fluxion::prelude::*;
use fluxion_exec::SubscribeLatestAsyncExt;
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use super::event_aggregation::create_aggregated_event;

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
        let (sensor_tx, sensor_rx) = mpsc::unbounded_channel();
        let (metrics_tx, metrics_rx) = mpsc::unbounded_channel();
        let (events_tx, events_rx) = mpsc::unbounded_channel();
        let (output_tx, output_rx) = mpsc::unbounded_channel();

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
        use futures::StreamExt;

        println!("🔄 Aggregator started\n");

        // Create intermediate channels for type-erased DataEvent streams
        let (data_tx1, data_rx1) = mpsc::unbounded_channel();
        let (data_tx2, data_rx2) = mpsc::unbounded_channel();
        let (data_tx3, data_rx3) = mpsc::unbounded_channel();

        // Transform domain events to DataEvent - intermediate channels provide type erasure
        let sensor_task = FluxionStream::from_unbounded_receiver(sensor_rx)
            .auto_ordered()
            .map_ordered(|s| DataEvent::Sensor(s.clone()))
            .for_each(move |event| {
                let _ = data_tx1.send(event);
                async {}
            });

        let metrics_task = FluxionStream::from_unbounded_receiver(metrics_rx)
            .auto_ordered()
            .map_ordered(|m| DataEvent::Metric(m.clone()))
            .for_each(move |event| {
                let _ = data_tx2.send(event);
                async {}
            });

        let events_task = FluxionStream::from_unbounded_receiver(events_rx)
            .auto_ordered()
            .map_ordered(|e| DataEvent::SystemEvent(e.clone()))
            .for_each(move |event| {
                let _ = data_tx3.send(event);
                async {}
            });

        // Combine the unified DataEvent streams
        let aggregation_task = FluxionStream::from_unbounded_receiver(data_rx1)
            .combine_latest(
                vec![
                    FluxionStream::from_unbounded_receiver(data_rx2),
                    FluxionStream::from_unbounded_receiver(data_rx3),
                ],
                |_| true,
            )
            .map_ordered(create_aggregated_event)
            .filter_ordered(|agg| {
                // Only process reasonable temperatures (150-300 represents 15.0-30.0°C)
                agg.get().temperature.map_or(false, |t| t >= 150 && t <= 300)
            })
            .subscribe_latest_async(
                move |agg, _token| {
                    let tx = output_tx.clone();
                    async move {
                        let temp_display = agg.temperature.map(|t| t as f64 / 10.0).unwrap_or(0.0);
                        let metric_display = agg.metric_value.map(|m| m as f64).unwrap_or(0.0);
                        
                        println!(
                            "\n  [Aggregator] @ {}: Temp={:.1}°C, Metric={:.1}, Alert={}",
                            agg.timestamp,
                            temp_display,
                            metric_display,
                            agg.has_alert
                        );
                        let _ = tx.send(agg);
                        Ok::<(), Infallible>(())
                    }
                },
                None::<fn(Infallible)>,
                Some(cancel_token),
            );

        // Run all transformation and aggregation tasks concurrently
        let _ = tokio::join!(sensor_task, metrics_task, events_task, aggregation_task);

        println!("\n🔄 Aggregator stopped");
    }
}
