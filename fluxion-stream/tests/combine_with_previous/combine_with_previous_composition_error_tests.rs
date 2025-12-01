// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{distinct_until_changed::DistinctUntilChangedExt, FluxionStream};
use fluxion_test_utils::{
    assert_no_element_emitted, person::Person, test_channel_with_errors, unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_error_propagation_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<Person>>();

    let mut result = FluxionStream::new(stream)
        .distinct_until_changed()
        .filter_ordered(|p| p.age >= 10)
        .combine_with_previous();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value.age == 10 && val.previous.is_none()
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Bob".to_string(), 20),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value.age == 20 && val.previous.as_ref().map(|p| p.value.age) == Some(10)
    ));

    drop(tx);

    Ok(())
}
