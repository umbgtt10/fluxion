// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![cfg(feature = "std")]

use fluxion_core::{FluxionError, Result, ResultExt};
use std::mem::size_of;

#[test]
fn test_error_display() {
    // Arrange
    let err = FluxionError::stream_error("processing failed");

    // Act
    let display = err.to_string();

    // Assert
    assert_eq!(display, "Stream processing error: processing failed");
}

#[test]
fn test_error_constructors() {
    // Arrange & Act
    let err = FluxionError::stream_error("processing failed");

    // Assert
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
}

#[test]
fn test_is_recoverable() {
    // Arrange & Act
    let stream_err = FluxionError::stream_error("test");
    let timeout_err = FluxionError::timeout_error("test");

    // Assert
    assert!(!stream_err.is_recoverable());
    assert!(!timeout_err.is_recoverable());
}

#[test]
fn test_is_permanent() {
    // Arrange & Act
    let stream_err = FluxionError::stream_error("test");
    let timeout_err = FluxionError::timeout_error("test");

    // Assert
    assert!(stream_err.is_permanent());
    assert!(!timeout_err.is_permanent());
}

#[test]
fn test_result_context_ok() {
    // Arrange
    let result: Result<i32> = Ok(42);

    // Act
    let value = result.context("operation failed").unwrap();

    // Assert
    assert_eq!(value, 42);
}

#[test]
fn test_with_context_ok() {
    // Arrange
    let result: Result<i32> = Ok(42);

    // Act
    let value = result
        .with_context(|| "should not be called".to_string())
        .unwrap();

    // Assert
    assert_eq!(value, 42);
}

#[test]
fn test_clone_stream_error() {
    // Arrange
    let err = FluxionError::stream_error("test stream");

    // Act
    let cloned = err.clone();

    // Assert
    assert!(matches!(cloned, FluxionError::StreamProcessingError { .. }));
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_timeout_error_constructor() {
    // Arrange & Act
    let err = FluxionError::timeout_error("Operation took too long");

    // Assert
    assert!(matches!(err, FluxionError::TimeoutError { .. }));
    assert_eq!(err.to_string(), "Timeout error: Operation took too long");
}

#[test]
fn test_timeout_error_display() {
    // Arrange
    let err = FluxionError::timeout_error("No data within 5 seconds");

    // Act
    let display = err.to_string();

    // Assert
    assert!(display.contains("Timeout error"));
    assert!(display.contains("No data within 5 seconds"));
}

#[test]
fn test_timeout_error_not_permanent() {
    // Arrange & Act
    let err = FluxionError::timeout_error("timeout");

    // Assert
    assert!(!err.is_permanent());
}

#[test]
fn test_timeout_error_not_recoverable() {
    // Arrange & Act
    let err = FluxionError::timeout_error("timeout");

    // Assert
    assert!(!err.is_recoverable());
}

#[test]
fn test_clone_timeout_error() {
    // Arrange
    let err = FluxionError::timeout_error("test timeout");

    // Act
    let cloned = err.clone();

    // Assert
    assert!(matches!(cloned, FluxionError::TimeoutError { .. }));
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_stream_error_with_empty_string() {
    // Arrange & Act
    let err = FluxionError::stream_error("");

    // Assert
    assert_eq!(err.to_string(), "Stream processing error: ");
}

#[test]
fn test_timeout_error_with_empty_string() {
    // Arrange & Act
    let err = FluxionError::timeout_error("");

    // Assert
    assert_eq!(err.to_string(), "Timeout error: ");
}

#[test]
fn test_context_with_timeout_error() {
    // Arrange
    let result: Result<()> = Err(FluxionError::timeout_error("no response"));

    // Act
    let err = result.context("operation timed out").unwrap_err();

    // Assert
    assert!(matches!(err, FluxionError::TimeoutError { .. }));
}

#[test]
fn test_debug_formatting() {
    // Arrange
    let err = FluxionError::stream_error("debug test");

    // Act
    let debug_str = format!("{:?}", err);

    // Assert
    assert!(debug_str.contains("StreamProcessingError"));
}

#[test]
fn test_error_type_sizes() {
    // Arrange & Act
    let error_size = size_of::<FluxionError>();

    // Assert
    assert!(error_size < 128);
}

#[test]
fn test_stream_error_creation_from_string() {
    // Arrange
    let msg = String::from("dynamic error");

    // Act
    let error = FluxionError::stream_error(msg);

    // Assert
    match error {
        FluxionError::StreamProcessingError { context } => {
            assert_eq!(context, "dynamic error");
        }
        _ => panic!("Expected StreamProcessingError"),
    }
}

#[test]
fn test_clone_preserves_context() {
    // Arrange
    let error = FluxionError::stream_error("original context");

    // Act
    let cloned = error.clone();

    // Assert
    match (error, cloned) {
        (
            FluxionError::StreamProcessingError { context: c1 },
            FluxionError::StreamProcessingError { context: c2 },
        ) => {
            assert_eq!(c1, c2);
            assert_eq!(c1, "original context");
        }
        _ => panic!("Expected StreamProcessingError for both"),
    }
}

#[test]
fn test_result_context_adds_nested_context() {
    // Arrange
    let result: Result<()> = Err(FluxionError::stream_error("inner"));

    // Act
    let with_context = result.context("outer");

    // Assert
    match with_context {
        Err(FluxionError::StreamProcessingError { context }) => {
            assert_eq!(context, "outer: inner");
        }
        _ => panic!("Expected StreamProcessingError with nested context"),
    }
}

#[test]
fn test_with_context_lazy_evaluation() {
    // Arrange
    let result: Result<()> = Err(FluxionError::stream_error("base"));
    let mut called = false;

    // Act
    let with_context = result.with_context(|| {
        called = true;
        String::from("lazy")
    });

    // Assert
    assert!(called);
    match with_context {
        Err(FluxionError::StreamProcessingError { context }) => {
            assert_eq!(context, "lazy: base");
        }
        _ => panic!("Expected StreamProcessingError"),
    }
}

#[test]
fn test_with_context_not_called_on_ok() {
    // Arrange
    let result: Result<i32> = Ok(100);
    let mut called = false;

    // Act
    let with_context = result.with_context(|| {
        called = true;
        String::from("should not be called")
    });

    // Assert
    assert!(!called);
    assert_eq!(with_context.unwrap(), 100);
}

#[test]
fn test_multiple_context_chaining() {
    // Arrange
    let result: Result<()> = Err(FluxionError::stream_error("root"));

    // Act
    let chained = result.context("level1").context("level2");

    // Assert
    match chained {
        Err(FluxionError::StreamProcessingError { context }) => {
            assert_eq!(context, "level2: level1: root");
        }
        _ => panic!("Expected nested context"),
    }
}

#[test]
fn test_context_from_different_string_types() {
    // Arrange
    let result1: Result<()> = Err(FluxionError::stream_error("error"));
    let result2: Result<()> = Err(FluxionError::stream_error("error"));

    // Act
    let with_str = result1.context("string slice");
    let with_string = result2.context(String::from("owned string"));

    // Assert
    assert!(matches!(
        with_str,
        Err(FluxionError::StreamProcessingError { .. })
    ));
    assert!(matches!(
        with_string,
        Err(FluxionError::StreamProcessingError { .. })
    ));
}

#[test]
fn test_error_trait_implementation() {
    // Arrange & Act
    let error = FluxionError::stream_error("test error");

    // Assert
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_display_formats_correctly() {
    // Arrange
    let stream_err = FluxionError::stream_error("connection lost");
    let timeout_err = FluxionError::timeout_error("5 seconds exceeded");

    // Act & Assert
    assert_eq!(
        format!("{}", stream_err),
        "Stream processing error: connection lost"
    );
    assert_eq!(
        format!("{}", timeout_err),
        "Timeout error: 5 seconds exceeded"
    );
}
