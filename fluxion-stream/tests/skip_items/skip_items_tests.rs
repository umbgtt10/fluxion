// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::SkipItemsExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    Sequenced,
};

#[tokio::test]
async fn test_skip_skips_initial_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();
    let mut result = stream.skip_items(2);

    // Act
    tx.send(Sequenced::new(person_alice()))?; // Skipped
    tx.send(Sequenced::new(person_bob()))?; // Skipped
    tx.send(Sequenced::new(person_charlie()))?; // Emitted
    drop(tx);

    // Assert - Only third item emitted
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_skip_more_than_available() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();
    let mut result = stream.skip_items(10);

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(person_bob()))?;
    drop(tx);

    // Assert - No items emitted (all skipped)
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
