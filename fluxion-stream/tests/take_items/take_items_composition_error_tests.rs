// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{MapOrderedExt, TakeItemsExt};
use fluxion_test_utils::{
    assert_stream_ended, test_channel_with_errors,
    test_data::{person_alice, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_map_ordered_then_take_items_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Map then take 2 items
    let mut result = stream.map_ordered(|x| x).take_items(2);

    // Act & Assert
    // 1. Send Alice -> Emitted (1/2)
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // 2. Send Error -> Should be propagated (2/2)
    // Note: Errors count as items in take_items
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // 3. Send another value -> Should NOT be emitted (limit reached)
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
