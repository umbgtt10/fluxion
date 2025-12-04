// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    test_channel_with_errors,
    test_data::{person_alice, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_map_ordered_then_distinct_until_changed_by_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Map to same value (identity) then distinct by age equality
    let mut result = FluxionStream::new(stream)
        .map_ordered(|x| x)
        .distinct_until_changed_by(|a, b| match (a, b) {
            (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
            _ => false,
        });

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}
