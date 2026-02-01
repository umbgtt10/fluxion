// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `sample_ratio` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::SampleRatioExt;
use fluxion_test_utils::helpers::{
    assert_no_element_emitted, test_channel_with_errors, unwrap_stream, unwrap_value,
};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{person_alice, person_bob, person_charlie, TestData};

#[tokio::test]
async fn test_sample_ratio_passes_through_errors_with_ratio_one() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(1.0, 42);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_passes_through_errors_with_ratio_zero() -> anyhow::Result<()> {
    // Arrange - even with ratio 0, errors must pass through
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(0.0, 42);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ),);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "another error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("another error")
    ),);

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_multiple_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(0.5, 42);

    // Act
    let error1 = FluxionError::stream_error("error 1");
    let error2 = FluxionError::stream_error("error 2");
    let error3 = FluxionError::stream_error("error 3");

    tx.unbounded_send(StreamItem::Error(error1))?;
    tx.unbounded_send(StreamItem::Error(error2))?;
    tx.unbounded_send(StreamItem::Error(error3))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("error 1")
    ),);

    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("error 2")
    ),);

    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("error 3")
    ),);

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_error_preserves_message() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(1.0, 42);

    // Act
    let error_message = "specific error message";
    let error = FluxionError::stream_error(error_message);
    tx.unbounded_send(StreamItem::Error(error))?;

    // Assert
    let item = unwrap_stream(&mut result, 500).await;
    match item {
        StreamItem::Error(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains(error_message),
                "Error message should be preserved. Got: {}",
                msg
            );
        }
        StreamItem::Value(_) => panic!("Expected error, got value"),
    }

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_interleaved_errors_and_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(1.0, 42);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;

    // Assert
    assert!(unwrap_stream(&mut result, 500).await.is_error());

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;

    // Assert
    assert!(unwrap_stream(&mut result, 500).await.is_error());

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_error_on_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(0.5, 42);

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("lone error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("lone error")
    ));

    assert_no_element_emitted(&mut result, 500).await;

    Ok(())
}
