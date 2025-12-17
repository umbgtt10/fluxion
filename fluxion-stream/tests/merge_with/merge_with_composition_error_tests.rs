// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::MergedStream;
use fluxion_test_utils::{person::Person, test_channel_with_errors, unwrap_stream, Sequenced};
use futures::StreamExt;

#[tokio::test]
async fn test_merge_with_multiple_streams_error() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<Person>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<Person>>();

    let mut result = MergedStream::seed::<Sequenced<Person>>(Person::new("State".to_string(), 0))
        .merge_with(stream1, |item: Person, state: &mut Person| {
            state.age += item.age;
            state.clone()
        })
        .merge_with(stream2, |item: Person, state: &mut Person| {
            state.age += item.age;
            state.clone()
        });

    // Act & Assert
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 10
    ));

    tx2.send(StreamItem::Error(FluxionError::stream_error("Error2")))?;
    let StreamItem::Error(err) = result.next().await.unwrap() else {
        panic!("Expected Error");
    };
    assert_eq!(err.to_string(), "Stream processing error: Error2");

    tx1.send(StreamItem::Error(FluxionError::stream_error("Error1")))?;
    let StreamItem::Error(err) = result.next().await.unwrap() else {
        panic!("Expected Error");
    };
    assert_eq!(err.to_string(), "Stream processing error: Error1");

    tx2.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Bob".to_string(), 20),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 30
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}
