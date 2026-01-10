// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_multi::TakeItemsExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    Sequenced,
};

#[tokio::test]
async fn test_take_limits_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();
    let mut result = stream.take_items(2);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // This won't be emitted

    // Assert - Only first 2 items
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );

    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );

    // Stream should end after 2 items
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_zero_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();
    let mut result = stream.take_items(0);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert - No items emitted
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
