// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{TokioTimer, TokioTimestamped};
use fluxion_test_utils::{
    helpers::recv_timeout, person::Person, test_channel_with_errors, test_data::person_alice,
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::pause;
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_throttle_chained_with_map_error_propagation() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();

    // Map then Throttle
    let throttled = stream
        .map_ordered(|x| {
            let val = if let TestData::Person(p) = x.value {
                TestData::Person(Person::new(p.name, p.age * 2))
            } else {
                x.value
            };
            TokioTimestamped::new(val, x.timestamp)
        })
        .throttle(Duration::from_millis(100));

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            result_tx.send(item).unwrap();
        }
    });

    // Act & Assert
    let error = FluxionError::stream_error("test error");
    tx.send(StreamItem::Error(error.clone()))?;

    assert_eq!(
        recv_timeout(&mut result_rx, 1000)
            .await
            .unwrap()
            .err()
            .expect("Expected Error")
            .to_string(),
        error.to_string()
    );

    tx.send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000)
            .await
            .unwrap()
            .ok()
            .expect("Expected Value")
            .value,
        TestData::Person(Person::new("Alice".to_string(), 50))
    );

    Ok(())
}
