// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, Result, ResultExt};
use std::{io, mem::size_of};

#[test]
fn test_result_context_adds_information() {
    let result: Result<()> = Err(FluxionError::UserError("original error".into()));
    let err = result.context("while performing operation").unwrap_err();

    let error_msg = err.to_string();
    assert!(error_msg.contains("while performing operation"));
    assert!(error_msg.contains("original error"));
}

#[test]
fn test_error_classification_recoverable() {
    let lock_err = FluxionError::lock_error("test mutex");
    assert!(lock_err.is_recoverable());
    assert!(!lock_err.is_permanent());
}

#[test]
fn test_error_classification_permanent() {
    let stream_err = FluxionError::stream_error("stream closed");
    assert!(!stream_err.is_recoverable());
    assert!(stream_err.is_permanent());

    let user_err = FluxionError::user_error(io::Error::other("io error"));
    assert!(!user_err.is_recoverable());
    assert!(user_err.is_permanent());
}

#[test]
fn test_multiple_errors_aggregation() {
    let errors = vec![
        FluxionError::lock_error("mutex1"),
        FluxionError::lock_error("mutex2"),
        FluxionError::stream_error("processing failed"),
    ];

    let multi_error = FluxionError::MultipleErrors { count: 3, errors };

    // MultipleErrors is not classified as recoverable or permanent by default
    // It depends on the contained errors
    assert!(!multi_error.is_recoverable());
    assert!(!multi_error.is_permanent());
}

#[test]
fn test_stream_processing_error() {
    let stream_err = FluxionError::stream_error("failed to process item");
    // StreamProcessingError is permanent
    assert!(!stream_err.is_recoverable());
    assert!(stream_err.is_permanent());
    assert!(stream_err.to_string().contains("failed to process item"));
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
fn test_with_context_lazy_evaluation() {
    let result: Result<i32> = Ok(42);

    // Expensive context function should not be called on Ok
    let value = result
        .with_context(|| {
            panic!("This should not be called for Ok results");
        })
        .unwrap();

    assert_eq!(value, 42);
}

#[test]
fn test_with_context_on_error() {
    let result: Result<()> = Err(FluxionError::UserError("base error".into()));

    let err = result
        .with_context(|| "Additional context from closure".to_string())
        .unwrap_err();

    let error_msg = err.to_string();
    assert!(error_msg.contains("Additional context from closure"));
}

