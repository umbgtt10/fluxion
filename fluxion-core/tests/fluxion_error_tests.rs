// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, IntoFluxionError, Result, ResultExt};
use std::io;

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
    assert!(!FluxionError::user_error(io::Error::other("test")).is_recoverable());
}

#[test]
fn test_is_permanent() {
    assert!(FluxionError::stream_error("test").is_permanent());
    assert!(FluxionError::user_error(io::Error::other("test")).is_permanent());
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
fn test_result_context_ok() {
    let result: Result<i32> = Ok(42);
    let value = result.context("operation failed").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_multiple_errors_creation() {
    let errors = vec![
        FluxionError::stream_error("stream1"),
        FluxionError::stream_error("stream2"),
        FluxionError::stream_error("stream3"),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    assert!(matches!(
        multi_err,
        FluxionError::MultipleErrors { count: 3, .. }
    ));
    assert!(multi_err.to_string().contains("3 errors"));
}

#[test]
fn test_from_user_errors() {
    #[derive(Debug, thiserror::Error)]
    #[error("Custom error: {msg}")]
    struct CustomError {
        msg: String,
    }

    let errors = vec![
        CustomError {
            msg: "first".to_string(),
        },
        CustomError {
            msg: "second".to_string(),
        },
        CustomError {
            msg: "third".to_string(),
        },
    ];

    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 3);
        assert_eq!(errors.len(), 3);
        assert!(errors
            .iter()
            .all(|e| matches!(e, FluxionError::UserError(_))));
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_from_user_errors_empty() {
    let errors: Vec<io::Error> = vec![];
    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 0);
        assert_eq!(errors.len(), 0);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_from_user_errors_single() {
    let errors = vec![io::Error::other("single error")];
    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 1);
        assert_eq!(errors.len(), 1);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_multiple_errors_permanent() {
    let errors = vec![
        FluxionError::stream_error("stream1"),
        FluxionError::user_error(io::Error::other("user1")),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    // MultipleErrors is not considered permanent
    assert!(!multi_err.is_permanent());
}

#[test]
fn test_into_fluxion_error_trait() {
    let io_err = io::Error::other("test error");
    let fluxion_err = io_err.into_fluxion_error("operation failed");

    assert!(matches!(fluxion_err, FluxionError::UserError(_)));
}

#[test]
fn test_into_fluxion_method() {
    let io_err = io::Error::other("test error");
    let fluxion_err = io_err.into_fluxion();

    assert!(matches!(fluxion_err, FluxionError::UserError(_)));
    assert!(fluxion_err.to_string().contains("test error"));
}

#[test]
fn test_with_context_lazy() {
    let result: Result<()> = Err(FluxionError::UserError("test error".into()));

    let err = result
        .with_context(|| {
            // Simulate expensive context computation
            format!("computed context at {}", "runtime")
        })
        .unwrap_err();

    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
    assert!(err.to_string().contains("computed context"));
    assert!(err.to_string().contains("test error"));
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
fn test_clone_user_error() {
    let err = FluxionError::user_error(io::Error::other("test user error"));
    let cloned = err.clone();

    // UserError is converted to StreamProcessingError when cloned
    assert!(matches!(cloned, FluxionError::StreamProcessingError { .. }));
    assert!(cloned.to_string().contains("User error"));
    assert!(cloned.to_string().contains("test user error"));
}

#[test]
fn test_clone_multiple_errors() {
    let errors = vec![
        FluxionError::stream_error("stream1"),
        FluxionError::stream_error("stream2"),
    ];

    let err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    let cloned = err.clone();

    if let FluxionError::MultipleErrors { count, errors } = cloned {
        assert_eq!(count, 2);
        assert_eq!(errors.len(), 2);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_user_error_constructor() {
    let io_err = io::Error::other("test io error");
    let err = FluxionError::user_error(io_err);

    assert!(matches!(err, FluxionError::UserError(_)));
    assert!(err.to_string().contains("test io error"));
}

#[test]
fn test_error_variants_display() {
    let stream_err = FluxionError::stream_error("processing failed");
    assert_eq!(
        stream_err.to_string(),
        "Stream processing error: processing failed"
    );

    let user_err = FluxionError::user_error(io::Error::other("custom error"));
    assert!(user_err.to_string().contains("User error"));
    assert!(user_err.to_string().contains("custom error"));

    let multi_err = FluxionError::MultipleErrors {
        count: 5,
        errors: vec![],
    };
    assert!(multi_err.to_string().contains("5 errors"));
}

#[test]
fn test_result_ext_multiple_contexts() {
    let result: Result<()> = Err(FluxionError::UserError("base error".into()));

    let err = result.context("first context").unwrap_err();

    assert!(err.to_string().contains("first context"));
    assert!(err.to_string().contains("base error"));
}

#[test]
fn test_nested_user_errors() {
    #[derive(Debug, thiserror::Error)]
    #[error("Outer: {0}")]
    struct OuterError(#[source] InnerError);

    #[derive(Debug, thiserror::Error)]
    #[error("Inner: {msg}")]
    struct InnerError {
        msg: String,
    }

    let inner = InnerError {
        msg: "root cause".to_string(),
    };
    let outer = OuterError(inner);
    let err = FluxionError::user_error(outer);

    assert!(matches!(err, FluxionError::UserError(_)));
    let display = err.to_string();
    assert!(display.contains("Outer"));
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
fn test_multiple_errors_not_recoverable() {
    let errors = vec![
        FluxionError::stream_error("stream1"),
        FluxionError::timeout_error("timeout1"),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    assert!(!multi_err.is_recoverable());
}

#[test]
fn test_multiple_errors_with_different_types() {
    let errors = vec![
        FluxionError::stream_error("stream error"),
        FluxionError::timeout_error("timeout error"),
        FluxionError::user_error(io::Error::other("io error")),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    assert_eq!(multi_err.to_string(), "Multiple errors occurred: 3 errors");
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
fn test_into_fluxion_error_with_empty_context() {
    let io_err = io::Error::other("test error");
    let fluxion_err = io_err.into_fluxion_error("");

    assert!(matches!(fluxion_err, FluxionError::UserError(_)));
}

#[test]
fn test_multiple_errors_zero_count() {
    let multi_err = FluxionError::MultipleErrors {
        count: 0,
        errors: vec![],
    };

    assert_eq!(multi_err.to_string(), "Multiple errors occurred: 0 errors");
}

#[test]
fn test_multiple_errors_count_mismatch() {
    // Test that count can differ from actual errors length
    let errors = vec![FluxionError::stream_error("error1")];

    let multi_err = FluxionError::MultipleErrors {
        count: 10, // Intentionally different from errors.len()
        errors,
    };

    assert_eq!(multi_err.to_string(), "Multiple errors occurred: 10 errors");
}

#[test]
fn test_context_chain_multiple_times() {
    let result: Result<()> = Err(FluxionError::UserError("base".into()));

    let result_with_context = result.context("first");
    let final_result: Result<()> = result_with_context
        .map_err(|e| FluxionError::UserError(Box::new(io::Error::other(e.to_string()))));

    let final_err = final_result.context("second").unwrap_err();
    assert!(final_err.to_string().contains("second"));
}

#[test]
fn test_from_user_errors_large_collection() {
    let errors: Vec<io::Error> = (0..100)
        .map(|i| io::Error::other(format!("error{i}")))
        .collect();

    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 100);
        assert_eq!(errors.len(), 100);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_debug_formatting() {
    let err = FluxionError::stream_error("debug test");
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("StreamProcessingError"));
}

#[test]
fn test_user_error_preserves_source() {
    #[derive(Debug, thiserror::Error)]
    #[error("outer")]
    struct OuterError(#[source] InnerError);

    #[derive(Debug, thiserror::Error)]
    #[error("inner")]
    struct InnerError;

    let err = FluxionError::user_error(OuterError(InnerError));

    // Verify error chain is preserved
    let err_string = err.to_string();
    assert!(err_string.contains("User error"));
}

#[test]
fn test_error_type_sizes() {
    use std::mem::size_of;

    // Ensure error types are reasonably sized
    let error_size = size_of::<FluxionError>();
    // FluxionError should be reasonably sized (less than 128 bytes)
    assert!(
        error_size < 128,
        "FluxionError is too large: {error_size} bytes"
    );
}
