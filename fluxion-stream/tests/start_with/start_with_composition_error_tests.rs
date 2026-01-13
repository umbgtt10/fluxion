// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{MapOrderedExt, StartWithExt};
use fluxion_test_utils::{
    test_channel_with_errors,
    test_data::{person_alice, person_bob, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_map_ordered_then_start_with_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Map then start with Bob
    let mut result = stream
        .map_ordered(|x| x)
        .start_with(vec![StreamItem::Value(Sequenced::new(person_bob()))]);

    // Act & Assert
    // 1. Should emit Bob first (from start_with)
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.clone().into_inner() == person_bob()
    ));

    // 2. Send Alice -> Should be emitted
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.clone().into_inner() == person_alice()
    ));

    // 3. Send Error -> Should be propagated
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}
