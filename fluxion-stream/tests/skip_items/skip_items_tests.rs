// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::SkipItemsExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, test_channel, unwrap_stream},
    sequenced::Sequenced,
    test_data::{person_alice, person_bob, person_charlie},
};

#[tokio::test]
async fn test_skip_skips_initial_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();
    let mut result = stream.skip_items(2);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    drop(tx);

    // Assert
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
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    drop(tx);

    // Assert
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
