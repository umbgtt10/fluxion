// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    Sequenced,
};

#[tokio::test]
async fn test_start_with_prepends_initial_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![Sequenced::new(person_alice()), Sequenced::new(person_bob())];

    let mut result =
        FluxionStream::new(stream).start_with(initial.into_iter().map(StreamItem::Value).collect());

    // Act & Assert - Initial values emitted first
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );

    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );

    // Now send from source stream
    tx.send(Sequenced::new(person_charlie()))?;

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
    let mut result = FluxionStream::new(stream).start_with(initial);

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    drop(tx);

    // Assert - Only source stream values emitted
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
