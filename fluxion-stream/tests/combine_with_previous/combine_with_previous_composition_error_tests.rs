// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{distinct_until_changed::DistinctUntilChangedExt, FluxionStream};
use fluxion_test_utils::{test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_distinct_until_changed_error_propagation_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Composition: distinct_until_changed -> filter -> combine_with_previous
    let mut result = FluxionStream::new(stream)
        .distinct_until_changed()
        .filter_ordered(|x| *x >= 0)
        .combine_with_previous();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == 1 && val.previous.is_none()
    ));

    // Send duplicate (filtered by distinct)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue - state should be preserved
    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == 2 && val.previous.as_ref().map(|p| p.value) == Some(1)
    ));

    drop(tx);

    Ok(())
}
