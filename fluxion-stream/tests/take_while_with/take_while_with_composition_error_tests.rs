// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};

use fluxion_stream::{MapOrderedExt, TakeWhileExt};
use fluxion_test_utils::{person::Person, test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_take_while_with_error_propagation_at_end_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<Person>>();
    let (condition_tx, condition_stream) = test_channel_with_errors::<Sequenced<bool>>();

    // Chain: map_ordered -> take_while_with
    // We map the source stream (increment age), then take while condition is true
    let mut result = source_stream
        .map_ordered(|seq| {
            let ts = HasTimestamp::timestamp(&seq);
            let mut p = seq.into_inner();
            p.age += 1;
            Sequenced::with_timestamp(p, ts)
        })
        .take_while_with(condition_stream, |cond| *cond);

    // Act & Assert
    condition_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(true, 0)))?;
    condition_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(false, 3)))?;

    // Act & Assert
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 11
    ));

    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Bob".to_string(), 20),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 21
    ));

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Charlie".to_string(), 30),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 31
    ));

    drop(source_tx);
    drop(condition_tx);

    Ok(())
}
