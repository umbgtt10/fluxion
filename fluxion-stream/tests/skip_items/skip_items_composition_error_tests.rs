// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{MapOrderedExt, SkipItemsExt};
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors,
    test_data::{person_alice, person_bob, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_map_ordered_then_skip_items_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Map then skip 1
    let mut result = stream.map_ordered(|x| x).skip_items(1);

    // Act
    // 1. Send Alice -> Skipped
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 2. Send Error -> Should be propagated (not skipped, as it's the 2nd item)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Map error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // 3. Send Bob -> Emitted
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}
