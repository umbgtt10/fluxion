// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Tests for error handling in composed pipelines with `on_error`.
//!
//! These tests verify that `on_error` correctly handles errors from upstream
//! operators in various composition scenarios.

use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream_multi::prelude::*;
use fluxion_stream_multi::{CombineLatestExt, OrderedStreamExt, TakeLatestWhenExt};
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel_with_errors,
    test_data::{animal_cat, animal_dog, person_alice, person_bob, person_charlie, TestData},
    unwrap_stream, unwrap_value, Sequenced,
};
use std::sync::Arc;

#[tokio::test]
async fn test_on_error_handles_errors_from_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let handled_errors = Arc::new(Mutex::new(Vec::new()));
    let handled_errors_clone = handled_errors.clone();

    let mut result = stream1.ordered_merge(vec![stream2]).on_error(move |err| {
        handled_errors_clone.lock().push(err.to_string());
        true // Consume all errors
    });

    // Act & Assert
    // 1. Send values from both streams
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&val.value, &person_alice());

    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        2,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&val.value, &person_bob());

    // 2. Send error from stream1
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "stream1 error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 3. Send error from stream2
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "stream2 error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 4. Verify both errors were handled
    {
        let errors = handled_errors.lock();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("stream1 error"));
        assert!(errors[1].contains("stream2 error"));
    }

    // 5. Continue after errors
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        3,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&val.value, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_on_error_handles_errors_from_combine_latest() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let handled_errors = Arc::new(Mutex::new(Vec::new()));
    let handled_errors_clone = handled_errors.clone();

    let mut result = stream1
        .combine_latest(vec![stream2], |_| true)
        .on_error(move |err| {
            handled_errors_clone.lock().push(err.to_string());
            true // Consume all errors
        });

    // Act & Assert
    // 1. Initialize both streams
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.values(), &[10, 20]);

    // 2. Send error from stream1
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "combine error 1",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 3. Verify error was handled
    assert_eq!(handled_errors.lock().len(), 1);

    // 4. Continue after error - stream still works
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.values(), &[30, 20]);

    Ok(())
}

#[tokio::test]
async fn test_on_error_handles_errors_from_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let source_errors = Arc::new(Mutex::new(0));
    let trigger_errors = Arc::new(Mutex::new(0));
    let source_errors_clone = source_errors.clone();
    let trigger_errors_clone = trigger_errors.clone();

    let mut result = source_stream
        .take_latest_when(
            trigger_stream,
            |t| matches!(t, TestData::Animal(a) if a.legs > 2),
        )
        .on_error(move |err| {
            let msg = err.to_string();
            if msg.contains("source") {
                *source_errors_clone.lock() += 1;
            } else if msg.contains("trigger") {
                *trigger_errors_clone.lock() += 1;
            }
            true // Consume all errors
        });

    // Act & Assert
    // 1. Buffer source value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 2. Trigger emission
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(&val.value, TestData::Person(p) if p.name == "Alice"));

    // 3. Send source error
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "source error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*source_errors.lock(), 1);

    // 4. Send trigger error
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "trigger error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*trigger_errors.lock(), 1);

    // 5. Continue after errors
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_cat(),
        4,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(&val.value, TestData::Person(p) if p.name == "Bob"));

    Ok(())
}

#[tokio::test]
async fn test_on_error_selective_handling_in_composed_pipeline() -> anyhow::Result<()> {
    // Arrange: Complex pipeline with selective error handling
    // source -> map_ordered -> on_error(handles "transient") -> filter_ordered -> on_error(handles "validation")
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let transient_errors = Arc::new(Mutex::new(0));
    let validation_errors = Arc::new(Mutex::new(0));
    let transient_clone = transient_errors.clone();
    let validation_clone = validation_errors.clone();

    let mut result = stream
        .map_ordered(|x| {
            // Add " mapped" suffix to person names
            if let TestData::Person(p) = &x.value {
                Sequenced::with_timestamp(
                    TestData::Person(fluxion_test_utils::person::Person {
                        name: format!("{} mapped", p.name),
                        age: p.age,
                    }),
                    x.timestamp(),
                )
            } else {
                x.clone()
            }
        })
        .on_error(move |err| {
            // Handle transient errors after map
            if err.to_string().contains("transient") {
                *transient_clone.lock() += 1;
                true
            } else {
                false
            }
        })
        .filter_ordered(|t| {
            // Only pass through Person types
            matches!(t, TestData::Person(_))
        })
        .on_error(move |err| {
            // Handle validation errors after filter
            if err.to_string().contains("validation") {
                *validation_clone.lock() += 1;
                true
            } else {
                false // Propagate other errors
            }
        });

    // Act & Assert
    // 1. Send valid value
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(&val.value, TestData::Person(p) if p.name == "Alice mapped"));

    // 2. Send transient error - handled by first on_error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "transient network issue",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*transient_errors.lock(), 1);
    assert_eq!(*validation_errors.lock(), 0);

    // 3. Send validation error - propagates past first, handled by second
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "validation failed",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*transient_errors.lock(), 1);
    assert_eq!(*validation_errors.lock(), 1);

    // 4. Send unhandled error - propagates through both handlers
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "critical system failure",
    )))?;
    // This error should propagate to the output
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("critical")
    ));

    // 5. Continue after propagated error
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(&val.value, TestData::Person(p) if p.name == "Bob mapped"));

    Ok(())
}

#[tokio::test]
async fn test_on_error_before_ordered_merge_handles_individual_stream_errors() -> anyhow::Result<()>
{
    // Arrange: Each stream has its own error handler before merging
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<TestData>>();

    let stream1_errors = Arc::new(Mutex::new(0));
    let stream2_errors = Arc::new(Mutex::new(0));
    let s1_errors_clone = stream1_errors.clone();
    let s2_errors_clone = stream2_errors.clone();

    // Apply on_error to each stream before merging
    let handled_stream1 = stream1.on_error(move |_err| {
        *s1_errors_clone.lock() += 1;
        true // Consume stream1 errors
    });

    let handled_stream2 = stream2.on_error(move |_err| {
        *s2_errors_clone.lock() += 1;
        true // Consume stream2 errors
    });

    let mut result = handled_stream1.ordered_merge(vec![handled_stream2]);

    // Act & Assert
    // 1. Send values
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&val.value, &person_alice());

    // 2. Send error to stream1 - handled by stream1's on_error
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("s1 error")))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*stream1_errors.lock(), 1);
    assert_eq!(*stream2_errors.lock(), 0);

    // 3. Send error to stream2 - handled by stream2's on_error
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("s2 error")))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*stream1_errors.lock(), 1);
    assert_eq!(*stream2_errors.lock(), 1);

    // 4. Continue normally
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        2,
    )))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&val.value, &person_bob());

    Ok(())
}

#[tokio::test]
async fn test_on_error_with_scan_ordered_preserves_accumulator_state() -> anyhow::Result<()> {
    // Arrange: scan_ordered accumulates state, on_error handles errors without affecting state
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let errors_handled = Arc::new(Mutex::new(0));
    let errors_clone = errors_handled.clone();

    let mut result = stream
        .scan_ordered(0i32, |acc, value| {
            *acc += value;
            *acc
        })
        .on_error(move |_err| {
            *errors_clone.lock() += 1;
            true // Consume errors
        });

    // Act & Assert
    // 1. Accumulate: 0 + 10 = 10
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    let val: Sequenced<i32> = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.value, 10);

    // 2. Accumulate: 10 + 20 = 30
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.value, 30);

    // 3. Error - should be consumed, state preserved
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("scan error")))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*errors_handled.lock(), 1);

    // 4. Continue accumulating: 30 + 5 = 35 (state was preserved!)
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.value, 35);

    // 5. Another error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "another error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*errors_handled.lock(), 2);

    // 6. Continue: 35 + 15 = 50
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(15, 4)))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.value, 50);

    Ok(())
}

#[tokio::test]
async fn test_on_error_with_combine_with_previous_maintains_previous_value() -> anyhow::Result<()> {
    // Arrange: combine_with_previous tracks previous value, on_error handles errors
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let errors_handled = Arc::new(Mutex::new(0));
    let errors_clone = errors_handled.clone();

    let mut result = stream.combine_with_previous().on_error(move |_err| {
        *errors_clone.lock() += 1;
        true
    });

    // Act & Assert
    // 1. First value - no previous
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(val.previous.is_none());
    assert!(matches!(&val.current.value, TestData::Person(p) if p.name == "Alice"));

    // 2. Second value - has previous (Alice)
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(&val.previous.unwrap().value, TestData::Person(p) if p.name == "Alice"));
    assert!(matches!(&val.current.value, TestData::Person(p) if p.name == "Bob"));

    // 3. Error - consumed, previous value should be preserved
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "combine error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*errors_handled.lock(), 1);

    // 4. Third value - previous should still be Bob (preserved through error)
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(&val.previous.unwrap().value, TestData::Person(p) if p.name == "Bob"));
    assert!(matches!(&val.current.value, TestData::Person(p) if p.name == "Charlie"));

    Ok(())
}

#[tokio::test]
async fn test_on_error_propagation_stops_at_first_handler() -> anyhow::Result<()> {
    // Arrange: Multiple on_error handlers - error stops at first matching handler
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let handler1_count = Arc::new(Mutex::new(0));
    let handler2_count = Arc::new(Mutex::new(0));
    let handler3_count = Arc::new(Mutex::new(0));
    let h1 = handler1_count.clone();
    let h2 = handler2_count.clone();
    let h3 = handler3_count.clone();

    let mut result = stream
        .on_error(move |err| {
            *h1.lock() += 1;
            // Only handle errors containing "type1"
            err.to_string().contains("type1")
        })
        .map_ordered(|x| x)
        .on_error(move |err| {
            *h2.lock() += 1;
            // Only handle errors containing "type2"
            err.to_string().contains("type2")
        })
        .filter_ordered(|_| true)
        .on_error(move |_err| {
            *h3.lock() += 1;
            // Catch-all handler
            true
        });

    // Act & Assert
    // 1. type1 error - handled by first handler only
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("type1 error")))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*handler1_count.lock(), 1);
    assert_eq!(*handler2_count.lock(), 0);
    assert_eq!(*handler3_count.lock(), 0);

    // 2. type2 error - passes through first, handled by second
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("type2 error")))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*handler1_count.lock(), 2); // Saw it but didn't handle
    assert_eq!(*handler2_count.lock(), 1); // Handled it
    assert_eq!(*handler3_count.lock(), 0);

    // 3. type3 error - passes through first two, handled by catch-all
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("type3 error")))?;
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(*handler1_count.lock(), 3); // Saw it
    assert_eq!(*handler2_count.lock(), 2); // Saw it
    assert_eq!(*handler3_count.lock(), 1); // Handled it

    // 4. Verify stream still works
    tx.unbounded_send(StreamItem::Value(Sequenced::new(42)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        42
    );

    Ok(())
}

#[tokio::test]
async fn test_on_error_handles_rapid_consecutive_errors_in_pipeline() -> anyhow::Result<()> {
    // Arrange: Test that rapid consecutive errors are all handled correctly
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let errors_handled = Arc::new(Mutex::new(0));
    let errors_clone = errors_handled.clone();

    let mut result = stream
        .map_ordered(|x| Sequenced::with_timestamp(x.value * 2, x.timestamp()))
        .on_error(move |_err| {
            *errors_clone.lock() += 1;
            true
        });

    // Act - send errors one at a time, polling after each
    for _i in 0..5 {
        tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;
        assert_no_element_emitted(&mut result, 50).await;
    }

    // Assert all errors were handled
    assert_eq!(*errors_handled.lock(), 5);

    // Verify stream still works after rapid errors
    tx.unbounded_send(StreamItem::Value(Sequenced::new(5)))?;
    let val: Sequenced<i32> = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(val.value, 10); // 5 * 2 from map_ordered

    Ok(())
}

#[tokio::test]
async fn test_on_error_empty_pipeline_with_errors() -> anyhow::Result<()> {
    // Arrange: Pipeline that only receives errors, no values
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let errors_count = Arc::new(Mutex::new(0));
    let errors_clone = errors_count.clone();

    let mut result = stream.filter_ordered(|_| true).on_error(move |_| {
        *errors_clone.lock() += 1;
        true
    });

    // Act - send errors one at a time, polling after each
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    assert_no_element_emitted(&mut result, 50).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    assert_no_element_emitted(&mut result, 50).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 3")))?;
    assert_no_element_emitted(&mut result, 50).await;

    // Assert
    assert_eq!(*errors_count.lock(), 3);

    // Close stream
    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}
