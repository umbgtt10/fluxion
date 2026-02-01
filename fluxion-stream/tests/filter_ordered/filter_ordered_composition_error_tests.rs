// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::{DistinctUntilChangedByExt, FilterOrderedExt, MapOrderedExt, ScanOrderedExt};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_by_multiple_errors_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream
        .distinct_until_changed_by(|a, b| a % 2 == b % 2)
        .map_ordered(|s| {
            let doubled = s.value * 2;
            Sequenced::new(doubled)
        })
        .filter_ordered(|x| *x > 0);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_propagation_with_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let accumulator = |count: &mut i32, _value: &i32| {
        *count += 1;
        *count
    };

    let mut result = stream
        .scan_ordered(0, accumulator)
        .filter_ordered(|count| count % 2 == 0);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(200, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(300, 4)))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(400, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 4
    ));

    Ok(())
}
