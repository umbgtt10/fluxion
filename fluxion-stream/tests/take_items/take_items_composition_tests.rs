// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave},
    Sequenced,
};

#[tokio::test]
async fn test_start_with_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(person_alice())),
        StreamItem::Value(Sequenced::new(person_bob())),
    ];

    let mut result = FluxionStream::new(stream).start_with(initial).take_items(3); // Take 2 initial + 1 from stream

    // Act
    tx.send(Sequenced::new(person_charlie()))?;
    tx.send(Sequenced::new(person_dave()))?; // Should not be emitted
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_skip_items_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let mut result = FluxionStream::new(stream)
        .skip_items(2) // Skip first 2
        .take_items(2); // Then take next 2

    // Act
    tx.send(Sequenced::new(person_alice()))?; // Skipped
    tx.send(Sequenced::new(person_bob()))?; // Skipped
    tx.send(Sequenced::new(person_charlie()))?; // Taken
    tx.send(Sequenced::new(person_dave()))?; // Taken
    tx.send(Sequenced::new(person_alice()))?; // Not emitted (take limit reached)

    // Assert - Only charlie and dave
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_dave()
    );
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
