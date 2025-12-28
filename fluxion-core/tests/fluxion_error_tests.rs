// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![cfg(feature = "std")]

use fluxion_core::{FluxionError, Result, ResultExt};
use std::mem::size_of;

#[test]
fn test_error_display() {
    let err = FluxionError::stream_error("processing failed");
    assert_eq!(
        err.to_string(),
        "Stream processing error: processing failed"
    );
}

#[test]
fn test_error_constructors() {
    let err = FluxionError::stream_error("processing failed");
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
}

#[test]
fn test_is_recoverable() {
    assert!(!FluxionError::stream_error("test").is_recoverable());
    assert!(!FluxionError::timeout_error("test").is_recoverable());
}

#[test]
fn test_is_permanent() {
    assert!(FluxionError::stream_error("test").is_permanent());
    assert!(!FluxionError::timeout_error("test").is_permanent());
}

#[test]
fn test_result_context_ok() {
    let result: Result<i32> = Ok(42);
    let value = result.context("operation failed").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_with_context_ok() {
    let result: Result<i32> = Ok(42);
    let value = result
        .with_context(|| "should not be called".to_string())
        .unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_clone_stream_error() {
    let err = FluxionError::stream_error("test stream");
    let cloned = err.clone();

    assert!(matches!(cloned, FluxionError::StreamProcessingError { .. }));
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_timeout_error_constructor() {
    let err = FluxionError::timeout_error("Operation took too long");
    assert!(matches!(err, FluxionError::TimeoutError { .. }));
    assert_eq!(err.to_string(), "Timeout error: Operation took too long");
}

#[test]
fn test_timeout_error_display() {
    let err = FluxionError::timeout_error("No data within 5 seconds");
    assert!(err.to_string().contains("Timeout error"));
    assert!(err.to_string().contains("No data within 5 seconds"));
}

#[test]
fn test_timeout_error_not_permanent() {
    let err = FluxionError::timeout_error("timeout");
    assert!(!err.is_permanent());
}

#[test]
fn test_timeout_error_not_recoverable() {
    let err = FluxionError::timeout_error("timeout");
    assert!(!err.is_recoverable());
}

#[test]
fn test_clone_timeout_error() {
    let err = FluxionError::timeout_error("test timeout");
    let cloned = err.clone();

    assert!(matches!(cloned, FluxionError::TimeoutError { .. }));
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_stream_error_with_empty_string() {
    let err = FluxionError::stream_error("");
    assert_eq!(err.to_string(), "Stream processing error: ");
}

#[test]
fn test_timeout_error_with_empty_string() {
    let err = FluxionError::timeout_error("");
    assert_eq!(err.to_string(), "Timeout error: ");
}

#[test]
fn test_context_with_timeout_error() {
    let result: Result<()> = Err(FluxionError::timeout_error("no response"));

    let err = result.context("operation timed out").unwrap_err();
    // Timeout errors pass through without wrapping
    assert!(matches!(err, FluxionError::TimeoutError { .. }));
}

#[test]
fn test_debug_formatting() {
    let err = FluxionError::stream_error("debug test");
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("StreamProcessingError"));
}

#[test]
fn test_error_type_sizes() {
    // Ensure error types are reasonably sized
    let error_size = size_of::<FluxionError>();
    // FluxionError should be reasonably sized (less than 128 bytes)
    assert!(
        error_size < 128,
        "FluxionError is too large: {error_size} bytes"
    );
}

#[test]
fn test_stream_error_creation_from_string() {
    let msg = String::from("dynamic error");
    let error = FluxionError::stream_error(msg);
    match error {
        FluxionError::StreamProcessingError { context } => {
            assert_eq!(context, "dynamic error");
        }
        _ => panic!("Expected StreamProcessingError"),
    }
}

#[test]
fn test_clone_preserves_context() {
    let error = FluxionError::stream_error("original context");
    let cloned = error.clone();

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
    let result: Result<()> = Err(FluxionError::stream_error("inner"));
    let with_context = result.context("outer");

    match with_context {
        Err(FluxionError::StreamProcessingError { context }) => {
            assert_eq!(context, "outer: inner");
        }
        _ => panic!("Expected StreamProcessingError with nested context"),
    }
}

#[test]
fn test_with_context_lazy_evaluation() {
    let result: Result<()> = Err(FluxionError::stream_error("base"));
    let mut called = false;

    let with_context = result.with_context(|| {
        called = true;
        String::from("lazy")
    });

    assert!(called, "Closure should have been called");
    match with_context {
        Err(FluxionError::StreamProcessingError { context }) => {
            assert_eq!(context, "lazy: base");
        }
        _ => panic!("Expected StreamProcessingError"),
    }
}

#[test]
fn test_with_context_not_called_on_ok() {
    let result: Result<i32> = Ok(100);
    let mut called = false;

    let with_context = result.with_context(|| {
        called = true;
        String::from("should not be called")
    });

    assert!(!called, "Closure should not have been called for Ok result");
    assert_eq!(with_context.unwrap(), 100);
}

#[test]
fn test_multiple_context_chaining() {
    let result: Result<()> = Err(FluxionError::stream_error("root"));
    let chained = result.context("level1").context("level2");

    match chained {
        Err(FluxionError::StreamProcessingError { context }) => {
            assert_eq!(context, "level2: level1: root");
        }
        _ => panic!("Expected nested context"),
    }
}

#[test]
fn test_context_from_different_string_types() {
    let result: Result<()> = Err(FluxionError::stream_error("error"));

    // Test with &str
    let with_str = result.context("string slice");
    assert!(matches!(
        with_str,
        Err(FluxionError::StreamProcessingError { .. })
    ));

    // Test with String
    let result2: Result<()> = Err(FluxionError::stream_error("error"));
    let with_string = result2.context(String::from("owned string"));
    assert!(matches!(
        with_string,
        Err(FluxionError::StreamProcessingError { .. })
    ));
}

#[test]
fn test_error_trait_implementation() {
    let error = FluxionError::stream_error("test error");
    // Ensure it implements std::error::Error
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_display_formats_correctly() {
    let stream_err = FluxionError::stream_error("connection lost");
    let timeout_err = FluxionError::timeout_error("5 seconds exceeded");

    assert_eq!(
        format!("{}", stream_err),
        "Stream processing error: connection lost"
    );
    assert_eq!(
        format!("{}", timeout_err),
        "Timeout error: 5 seconds exceeded"
    );
}
