// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{MapOrderedExt, TakeItemsExt};
use fluxion_test_utils::{
    helpers::{assert_stream_ended, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::{person_alice, TestData},
};

#[tokio::test]
async fn test_map_ordered_then_take_items_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Map then take 2 items
    let mut result = stream.map_ordered(|x| x).take_items(2);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    // Note: Errors count as items in take_items
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Map error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
