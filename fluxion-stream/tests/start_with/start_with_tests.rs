// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::StartWithExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, test_channel, unwrap_stream},
    sequenced::Sequenced,
    test_data::{person_alice, person_bob, person_charlie},
};

#[tokio::test]
async fn test_start_with_prepends_initial_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![Sequenced::new(person_alice()), Sequenced::new(person_bob())];

    // Act
    let mut result = stream.start_with(initial.into_iter().map(StreamItem::Value).collect());

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_start_with_empty_initial_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![];
    let mut result = stream.start_with(initial);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
