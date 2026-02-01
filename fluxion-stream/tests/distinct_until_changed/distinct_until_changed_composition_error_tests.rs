// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{DistinctUntilChangedExt, MapOrderedExt};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream, unwrap_value},
    sequenced::Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_multiple_errors_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream
        .map_ordered(|s| {
            let doubled = s.value * 2;
            Sequenced::new(doubled)
        })
        .distinct_until_changed();

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?; // 2

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 4)))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 5)))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        4
    );

    drop(tx);

    Ok(())
}
