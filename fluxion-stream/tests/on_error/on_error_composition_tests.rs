// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{CombineWithPreviousExt, FilterOrderedExt, MapOrderedExt, OnErrorExt};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob, person_charlie, person_dave, TestData},
    unwrap_value, Sequenced,
};
use std::sync::Arc;

#[tokio::test]
async fn test_on_error_in_middle_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let early_errors = Arc::new(Mutex::new(0));
    let late_errors = Arc::new(Mutex::new(0));
    let early_errors_clone = early_errors.clone();
    let late_errors_clone = late_errors.clone();

    let mut stream = stream
        .on_error(move |err| {
            // Handle specific errors early
            if err.to_string().contains("early") {
                *early_errors_clone.lock() += 1;
                true // Consume early errors
            } else {
                false // Propagate other errors
            }
        })
        .combine_with_previous()
        .map_ordered(|with_prev| with_prev.current)
        .on_error(move |err| {
            // Handle remaining errors late
            if err.to_string().contains("late") {
                *late_errors_clone.lock() += 1;
                true // Consume late errors
            } else {
                false // Propagate unhandled errors
            }
        });

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_alice());

    // Send early error - should be consumed by first handler
    tx.try_send(StreamItem::Error(FluxionError::stream_error(
        "early error 1",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*early_errors.lock(), 1);
    assert_eq!(*late_errors.lock(), 0);

    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_bob());

    // Send late error - should be propagated and consumed by second handler
    tx.try_send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "late error 1",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*early_errors.lock(), 1);
    assert_eq!(*late_errors.lock(), 1);

    // Send another early error - should be consumed by first handler
    tx.try_send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "early error 2",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*early_errors.lock(), 2);
    assert_eq!(*late_errors.lock(), 1);

    tx.try_send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_charlie());

    // Send another late error - should be propagated and consumed by second handler
    tx.try_send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "late error 2",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(
        *early_errors.lock(),
        2,
        "Should have handled 2 early errors"
    );
    assert_eq!(*late_errors.lock(), 2, "Should have handled 2 late errors");

    Ok(())
}

#[tokio::test]
async fn test_on_error_chain_of_responsibility() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let network_errors = Arc::new(Mutex::new(0));
    let validation_errors = Arc::new(Mutex::new(0));
    let other_errors = Arc::new(Mutex::new(0));
    let network_errors_clone = network_errors.clone();
    let validation_errors_clone = validation_errors.clone();
    let other_errors_clone = other_errors.clone();

    let mut stream = stream
        .on_error(move |err| {
            // First handler: network errors
            if err.to_string().contains("network") {
                *network_errors_clone.lock() += 1;
                true
            } else {
                false
            }
        })
        .combine_with_previous()
        .on_error(move |err| {
            // Second handler: validation errors
            if err.to_string().contains("validation") {
                *validation_errors_clone.lock() += 1;
                true
            } else {
                false
            }
        })
        .map_ordered(|with_prev| with_prev.current)
        .filter_ordered(|_| true)
        .on_error(move |_err| {
            // Final catch-all handler
            *other_errors_clone.lock() += 1;
            true
        });

    // Act & Assert - send and verify each item with error handling at each stage
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_alice());

    // Send network error - consumed by first handler
    tx.try_send(StreamItem::Error(FluxionError::stream_error(
        "network timeout",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*network_errors.lock(), 1);
    assert_eq!(*validation_errors.lock(), 0);
    assert_eq!(*other_errors.lock(), 0);

    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_bob());

    // Send validation error - propagated past first, consumed by second handler
    tx.try_send(StreamItem::Error(FluxionError::stream_error(
        "validation failed",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*network_errors.lock(), 1);
    assert_eq!(*validation_errors.lock(), 1);
    assert_eq!(*other_errors.lock(), 0);

    tx.try_send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_charlie());

    // Send unknown error - propagated past first two, consumed by catch-all
    tx.try_send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "unknown error",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*network_errors.lock(), 1);
    assert_eq!(*validation_errors.lock(), 1);
    assert_eq!(*other_errors.lock(), 1);

    tx.try_send(StreamItem::Value(Sequenced::new(person_dave())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_dave());

    // Send another network error - consumed by first handler
    tx.try_send(StreamItem::Error(FluxionError::stream_error(
        "network connection lost",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(
        *network_errors.lock(),
        2,
        "Should have handled 2 network errors"
    );
    assert_eq!(
        *validation_errors.lock(),
        1,
        "Should have handled 1 validation error"
    );
    assert_eq!(
        *other_errors.lock(),
        1,
        "Should have handled 1 unknown error"
    );

    Ok(())
}
