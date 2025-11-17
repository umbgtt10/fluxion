// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_error::{FluxionError, Result, ResultExt};
use std::io;

#[test]
fn test_error_display() {
    let err = FluxionError::lock_error("test mutex");
    assert_eq!(err.to_string(), "Failed to acquire lock: test mutex");

    let err = FluxionError::stream_error("processing failed");
    assert_eq!(
        err.to_string(),
        "Stream processing error: processing failed"
    );
}

#[test]
fn test_error_constructors() {
    let err = FluxionError::lock_error("my lock");
    assert!(matches!(err, FluxionError::LockError { .. }));

    let err = FluxionError::stream_error("processing failed");
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
}

#[test]
fn test_is_recoverable() {
    assert!(FluxionError::lock_error("test").is_recoverable());
    assert!(!FluxionError::stream_error("test").is_recoverable());
    assert!(!FluxionError::user_error(io::Error::other("test")).is_recoverable());
}

#[test]
fn test_is_permanent() {
    assert!(FluxionError::stream_error("test").is_permanent());
    assert!(FluxionError::user_error(io::Error::other("test")).is_permanent());
    assert!(!FluxionError::lock_error("test").is_permanent());
}

#[test]
fn test_result_context() {
    // Create a result with a UserError which gets wrapped by context
    let result: Result<()> = Err(FluxionError::UserError("test error".into()));

    let err = result.context("operation failed").unwrap_err();
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
    assert!(err.to_string().contains("operation failed"));
    assert!(err.to_string().contains("test error"));
}

#[test]
fn test_result_context_preserves_non_user_errors() {
    // Other error types are preserved, not wrapped
    let result: Result<()> = Err(FluxionError::LockError {
        context: "test lock error".to_string(),
    });

    let err = result.context("operation failed").unwrap_err();
    assert!(matches!(err, FluxionError::LockError { .. }));
}

#[test]
fn test_result_context_ok() {
    let result: Result<i32> = Ok(42);
    let value = result.context("operation failed").unwrap();
    assert_eq!(value, 42);
}
