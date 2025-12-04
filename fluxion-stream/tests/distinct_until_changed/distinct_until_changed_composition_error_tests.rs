// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    assert_no_element_emitted, helpers::unwrap_value, test_channel_with_errors, unwrap_stream,
    Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_multiple_errors_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Composition: map -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let doubled = s.value * 2;
            Sequenced::new(doubled)
        })
        .distinct_until_changed();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?; // 2
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 4)))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 5)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        4
    );

    drop(tx);

    Ok(())
}
