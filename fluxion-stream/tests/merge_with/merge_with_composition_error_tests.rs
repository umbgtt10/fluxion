// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::MergedStream;
use fluxion_test_utils::{
    person::Person,
    test_channel_with_errors,
    test_data::{person_alice, person_bob, TestData},
    unwrap_stream, Sequenced,
};
use futures::StreamExt;

#[tokio::test]
async fn test_merge_with_multiple_streams_error() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result = MergedStream::seed::<Sequenced<Person>>(Person::new("State".to_string(), 0))
        .merge_with(stream1, |item: TestData, state: &mut Person| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .merge_with(stream2, |item: TestData, state: &mut Person| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx1.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref v) if v.value.age == 25)
    );

    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error2")))?;
    assert!(
        matches!(result.next().await.unwrap(), StreamItem::Error(ref e) if e.to_string() == "Stream processing error: Error2")
    );

    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error1")))?;
    assert!(
        matches!(result.next().await.unwrap(), StreamItem::Error(ref e) if e.to_string() == "Stream processing error: Error1")
    );

    tx2.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref v) if v.value.age == 55)
    );

    Ok(())
}
